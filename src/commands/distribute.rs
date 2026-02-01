use crate::appcast::{self, AppcastItem};
use crate::changelog;
use crate::config::Config;
use crate::error::{DmgrError, Result};
use crate::output;
use crate::runner;
use crate::targets::{github, s3};
use std::fs;
use std::path::{Path, PathBuf};

pub struct DistributeOptions {
    pub dmg_path: PathBuf,
    pub changelog_path: Option<PathBuf>,
    pub appcast_path: Option<PathBuf>,
    pub output_appcast_path: Option<PathBuf>,
    pub github_enabled: bool,
    pub github_repo: Option<String>,
    pub s3_enabled: bool,
    pub s3_bucket: Option<String>,
    pub s3_prefix: Option<String>,
    pub download_url_override: Option<String>,
    pub dry_run: bool,
    pub skip_changelog: bool,
}

/// App info extracted from the DMG
struct AppInfo {
    name: String,
    version: String,
    build: String,
    minimum_system_version: Option<String>,
}

pub fn run(opts: DistributeOptions) -> Result<()> {
    output::header("dmgr distribute");
    println!();

    // Load config
    let config = Config::load()?;

    // Validate DMG exists
    if !opts.dmg_path.exists() {
        return Err(DmgrError::DmgNotFound(
            opts.dmg_path.to_string_lossy().to_string(),
        ));
    }

    let dmg_size = fs::metadata(&opts.dmg_path)?.len();
    output::list_item("DMG", &opts.dmg_path.to_string_lossy());
    output::list_item("Size", &format!("{:.2} MB", dmg_size as f64 / 1_048_576.0));
    println!();

    // Step 1: Extract app info from DMG
    output::step("Extracting app info from DMG");
    let app_info = if opts.dry_run {
        AppInfo {
            name: "MyApp".to_string(),
            version: "1.0.0".to_string(),
            build: "100".to_string(),
            minimum_system_version: Some("10.15".to_string()),
        }
    } else {
        extract_app_info(&opts.dmg_path)?
    };
    output::list_item("App", &app_info.name);
    output::list_item("Version", &app_info.version);
    output::list_item("Build", &app_info.build);
    if let Some(ref min_ver) = app_info.minimum_system_version {
        output::list_item("Min macOS", min_ver);
    }
    println!();

    // Step 2: Parse changelog
    let changelog_entry = if opts.skip_changelog {
        output::info("Skipping changelog");
        None
    } else {
        output::step("Parsing changelog");
        let changelog_path = opts
            .changelog_path
            .or_else(|| config.distribution.changelog.as_ref().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("Changelog.md"));

        if !changelog_path.exists() {
            return Err(DmgrError::ChangelogNotFound(
                changelog_path.to_string_lossy().to_string(),
            ));
        }

        let entries = changelog::parse_changelog(&changelog_path)
            .map_err(DmgrError::ChangelogParseError)?;

        let entry = changelog::find_entry_for_version(&entries, &app_info.version)
            .ok_or_else(|| DmgrError::ChangelogEntryMissing(app_info.version.clone()))?;

        output::success(&format!(
            "Found changelog entry for {}",
            app_info.version
        ));
        Some(entry.clone())
    };
    println!();

    // Step 3: Load existing appcast if provided
    let mut existing_items: Vec<AppcastItem> = if let Some(ref path) = opts.appcast_path {
        output::step("Loading existing appcast");
        if path.exists() {
            let items = appcast::parse_appcast(path).map_err(DmgrError::AppcastError)?;
            output::success(&format!("Loaded {} existing items", items.len()));
            items
        } else {
            output::info("Appcast file not found, starting fresh");
            Vec::new()
        }
    } else {
        Vec::new()
    };
    if opts.appcast_path.is_some() {
        println!();
    }

    // Determine targets
    let github_enabled = opts.github_enabled
        || config.distribution.github.enabled.unwrap_or(false);
    let s3_enabled = opts.s3_enabled || config.distribution.s3.enabled.unwrap_or(false);

    if !github_enabled && !s3_enabled && opts.download_url_override.is_none() {
        return Err(DmgrError::NoDistributionTarget);
    }

    let mut github_url: Option<String> = None;
    let mut s3_url: Option<String> = None;

    // Step 4: Upload to GitHub
    if github_enabled {
        output::step("Uploading to GitHub");

        let repo = opts
            .github_repo
            .or_else(|| config.distribution.github.repo.clone())
            .ok_or(DmgrError::NoGitHubRepo)?;

        let tag_prefix = config
            .distribution
            .github
            .tag_prefix
            .as_deref()
            .unwrap_or("v");
        let tag = format!("{}{}", tag_prefix, app_info.version);

        let notes = changelog_entry.as_ref().map(|e| e.content_markdown.as_str());

        let result = github::upload_release(&repo, &tag, &opts.dmg_path, notes, opts.dry_run)
            .map_err(DmgrError::GitHubError)?;

        github_url = Some(result.download_url);
        println!();
    }

    // Step 5: Upload to S3
    if s3_enabled {
        output::step("Uploading to S3");

        let bucket = opts
            .s3_bucket
            .clone()
            .or_else(|| config.distribution.s3.bucket.clone())
            .ok_or(DmgrError::NoS3Bucket)?;

        let prefix = opts
            .s3_prefix
            .clone()
            .or_else(|| config.distribution.s3.prefix.clone());

        let region = config.distribution.s3.region.as_deref();

        let result = s3::upload_file(
            &bucket,
            prefix.as_deref(),
            &opts.dmg_path,
            region,
            opts.dry_run,
        )
        .map_err(DmgrError::S3Error)?;

        s3_url = Some(result.download_url);
        println!();
    }

    // Step 6: Determine final download URL
    let download_url = opts
        .download_url_override
        .or(github_url)
        .or(s3_url)
        .ok_or(DmgrError::NoDownloadUrl)?;

    output::list_item("Download URL", &download_url);
    println!();

    // Step 7: Sign DMG with EdDSA
    output::step("Signing DMG for Sparkle");
    let private_key_path = config.sparkle.private_key.as_ref().map(PathBuf::from);
    let ed_signature = if opts.dry_run {
        output::info("(dry run - skipping signing)");
        Some("fake-signature".to_string())
    } else {
        match appcast::sign_dmg(&opts.dmg_path, private_key_path.as_deref()) {
            Ok(sig) => {
                output::success("DMG signed");
                Some(sig)
            }
            Err(e) => {
                output::warning(&format!("Failed to sign DMG: {}", e));
                output::info("Appcast will be generated without EdDSA signature");
                None
            }
        }
    };
    println!();

    // Step 8: Create appcast item
    output::step("Generating appcast entry");
    let new_item = appcast::create_item(
        &app_info.name,
        &app_info.version,
        &app_info.build,
        &download_url,
        dmg_size,
        ed_signature,
        changelog_entry.as_ref(),
        app_info.minimum_system_version,
    );

    // Step 9: Merge into appcast
    appcast::merge_item(&mut existing_items, new_item).map_err(DmgrError::AppcastError)?;
    output::success(&format!(
        "Appcast now has {} items",
        existing_items.len()
    ));
    println!();

    // Step 10: Write appcast.xml
    output::step("Writing appcast.xml");
    let output_appcast_path = opts
        .output_appcast_path
        .or_else(|| config.sparkle.appcast_output.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("appcast.xml"));

    let appcast_url = config
        .sparkle
        .appcast_url
        .as_deref()
        .unwrap_or("https://example.com/appcast.xml");

    let appcast_content = appcast::generate_appcast(
        &app_info.name,
        &format!("{} release notes", app_info.name),
        appcast_url,
        &existing_items,
    )
    .map_err(DmgrError::AppcastError)?;

    if !opts.dry_run {
        appcast::write_appcast(&output_appcast_path, &appcast_content)
            .map_err(DmgrError::AppcastError)?;
    }
    output::success(&format!(
        "Wrote: {}",
        output_appcast_path.display()
    ));
    println!();

    // Step 11: Upload appcast to S3 (if S3 is enabled)
    if s3_enabled {
        output::step("Uploading appcast to S3");
        let bucket = opts
            .s3_bucket
            .as_ref()
            .or(config.distribution.s3.bucket.as_ref())
            .ok_or(DmgrError::NoS3Bucket)?;

        let prefix = opts
            .s3_prefix
            .as_ref()
            .or(config.distribution.s3.prefix.as_ref())
            .map(|s| s.as_str());

        let region = config.distribution.s3.region.as_deref();

        s3::upload_appcast(bucket, prefix, &output_appcast_path, region, opts.dry_run)
            .map_err(DmgrError::S3Error)?;
        println!();
    }

    output::success("Distribution complete!");

    Ok(())
}

/// Extract app info by mounting the DMG and reading Info.plist
fn extract_app_info(dmg_path: &Path) -> Result<AppInfo> {
    // Create a temporary mount point
    let mount_output =
        runner::run_capture("hdiutil", &["attach", "-nobrowse", "-readonly", "-plist", dmg_path.to_str().unwrap()])?;

    // Parse the plist output to find mount point
    let mount_point = parse_mount_point(&mount_output).ok_or(DmgrError::DmgMountFailed)?;

    // Find .app bundle
    let app_path = find_app_in_volume(&mount_point)?;

    // Extract version info
    let info_plist = app_path.join("Contents/Info.plist");
    let plist_str = info_plist.to_string_lossy();

    let name = runner::run_capture(
        "/usr/libexec/PlistBuddy",
        &["-c", "Print CFBundleName", &plist_str],
    )
    .map(|s| s.trim().to_string())
    .unwrap_or_else(|_| {
        // Fallback to executable name
        app_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    });

    let version = runner::run_capture(
        "/usr/libexec/PlistBuddy",
        &["-c", "Print CFBundleShortVersionString", &plist_str],
    )
    .map_err(|_| DmgrError::VersionExtractFailed)?
    .trim()
    .to_string();

    let build = runner::run_capture(
        "/usr/libexec/PlistBuddy",
        &["-c", "Print CFBundleVersion", &plist_str],
    )
    .map(|s| s.trim().to_string())
    .unwrap_or_else(|_| version.clone());

    let minimum_system_version = runner::run_capture(
        "/usr/libexec/PlistBuddy",
        &["-c", "Print LSMinimumSystemVersion", &plist_str],
    )
    .map(|s| s.trim().to_string())
    .ok();

    // Unmount the DMG
    let _ = runner::run_capture("hdiutil", &["detach", &mount_point]);

    Ok(AppInfo {
        name,
        version,
        build,
        minimum_system_version,
    })
}

/// Parse hdiutil plist output to find mount point
fn parse_mount_point(plist_output: &str) -> Option<String> {
    // Look for mount-point in the output
    // The plist output contains system-entities with mount-point
    for line in plist_output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<string>/Volumes/") {
            let start = trimmed.find("/Volumes/").unwrap();
            let end = trimmed.rfind("</string>").unwrap_or(trimmed.len());
            return Some(trimmed[start..end].to_string());
        }
    }

    // Fallback: try to find /Volumes path in any format
    for line in plist_output.lines() {
        if line.contains("/Volumes/") {
            if let Some(start) = line.find("/Volumes/") {
                let rest = &line[start..];
                let end = rest
                    .find(|c: char| c == '<' || c == '"' || c == '\n')
                    .unwrap_or(rest.len());
                return Some(rest[..end].to_string());
            }
        }
    }

    None
}

/// Find the .app bundle in a mounted volume
fn find_app_in_volume(mount_point: &str) -> Result<PathBuf> {
    let entries = fs::read_dir(mount_point)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("app") {
            return Ok(path);
        }
    }

    Err(DmgrError::NoAppInDmg)
}
