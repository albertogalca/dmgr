use crate::config::Config;
use crate::error::{DmgrError, Result};
use crate::output;
use crate::runner;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(
    scheme: String,
    build_config: String,
    output_dir: Option<String>,
    identity_override: Option<String>,
    notarize: bool,
    dry_run: bool,
) -> Result<()> {
    output::header(&format!("Building {}", scheme));
    println!();

    let config = Config::load()?;

    let identity = identity_override
        .or(config.signing.identity.clone())
        .or_else(|| get_first_identity().ok())
        .ok_or(DmgrError::NoSigningIdentity)?;

    output::list_item("Scheme", &scheme);
    output::list_item("Configuration", &build_config);
    output::list_item("Signing identity", &identity);
    if notarize {
        let profile = config
            .notarization
            .keychain_profile
            .as_ref()
            .ok_or(DmgrError::NoNotarizationProfile)?;
        output::list_item("Notarization profile", profile);
    }
    println!();

    let build_dir = PathBuf::from("build");
    let archive_path = build_dir.join(format!("{}.xcarchive", scheme));
    let export_path = build_dir.join("export");
    let output_path = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    output::step("Creating Xcode archive");
    xcodebuild_archive(&scheme, &build_config, &archive_path, dry_run)?;
    println!();

    output::step("Exporting archive");
    let export_plist = create_export_options_plist(&identity, &build_dir, dry_run)?;
    xcodebuild_export(&archive_path, &export_path, &export_plist, dry_run)?;
    println!();

    let app_path = export_path.join(format!("{}.app", scheme));

    output::step("Code signing");
    codesign_app(&app_path, &identity, dry_run)?;
    verify_signature(&app_path, dry_run)?;
    println!();

    let version = if dry_run {
        "1.0.0".to_string()
    } else {
        extract_version(&app_path)?
    };

    output::step("Creating DMG");
    let dmg_name = format!("{}-{}.dmg", scheme, version);
    let dmg_path = output_path.join(&dmg_name);
    create_dmg(&app_path, &dmg_path, &scheme, &config, dry_run)?;
    println!();

    output::step("Signing DMG");
    codesign_dmg(&dmg_path, &identity, dry_run)?;
    println!();

    if notarize {
        let profile = config
            .notarization
            .keychain_profile
            .as_ref()
            .ok_or(DmgrError::NoNotarizationProfile)?;

        output::step("Notarizing DMG");
        notarize_dmg(&dmg_path, profile, dry_run)?;
        println!();

        output::step("Stapling notarization ticket");
        staple_dmg(&dmg_path, dry_run)?;
        println!();

        output::step("Verifying notarization");
        verify_notarization(&dmg_path, dry_run)?;
        println!();
    }

    output::success(&format!("Created: {}", dmg_path.display()));

    if !dry_run {
        output::info("Cleaning up build artifacts...");
        let _ = fs::remove_dir_all(&build_dir);
    }

    Ok(())
}

fn get_first_identity() -> Result<String> {
    let output = runner::run_capture("security", &["find-identity", "-v", "-p", "codesigning"])?;

    for line in output.lines() {
        if line.contains("Developer ID Application:") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start < end {
                        return Ok(line[start + 1..end].to_string());
                    }
                }
            }
        }
    }

    Err(DmgrError::NoSigningIdentity)
}

fn xcodebuild_archive(
    scheme: &str,
    config: &str,
    archive_path: &Path,
    dry_run: bool,
) -> Result<()> {
    let archive_str = archive_path.to_string_lossy();
    let args = vec![
        "archive",
        "-scheme",
        scheme,
        "-configuration",
        config,
        "-archivePath",
        &archive_str,
    ];

    runner::run("xcodebuild", &args, dry_run)?;
    Ok(())
}

fn create_export_options_plist(identity: &str, build_dir: &Path, dry_run: bool) -> Result<PathBuf> {
    let plist_path = build_dir.join("ExportOptions.plist");

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key>
    <string>developer-id</string>
    <key>signingStyle</key>
    <string>manual</string>
    <key>signingCertificate</key>
    <string>{}</string>
</dict>
</plist>"#,
        identity
    );

    if !dry_run {
        fs::create_dir_all(build_dir)?;
        fs::write(&plist_path, plist_content)?;
    }

    output::info(&format!(
        "Created export options: {}",
        plist_path.display()
    ));
    Ok(plist_path)
}

fn xcodebuild_export(
    archive_path: &Path,
    export_path: &Path,
    export_plist: &Path,
    dry_run: bool,
) -> Result<()> {
    let archive_str = archive_path.to_string_lossy();
    let export_str = export_path.to_string_lossy();
    let plist_str = export_plist.to_string_lossy();

    let args = vec![
        "-exportArchive",
        "-archivePath",
        &archive_str,
        "-exportPath",
        &export_str,
        "-exportOptionsPlist",
        &plist_str,
    ];

    runner::run("xcodebuild", &args, dry_run)?;
    Ok(())
}

fn codesign_app(app_path: &Path, identity: &str, dry_run: bool) -> Result<()> {
    let app_str = app_path.to_string_lossy();

    let args = vec![
        "--force",
        "--deep",
        "--timestamp",
        "--options",
        "runtime",
        "--sign",
        identity,
        &app_str,
    ];

    runner::run("codesign", &args, dry_run)?;
    output::success("App signed");
    Ok(())
}

fn verify_signature(app_path: &Path, dry_run: bool) -> Result<()> {
    let app_str = app_path.to_string_lossy();
    let args = vec!["--verify", "--deep", "--strict", &app_str];

    runner::run("codesign", &args, dry_run)?;
    output::success("Signature verified");
    Ok(())
}

fn extract_version(app_path: &Path) -> Result<String> {
    let plist_path = app_path.join("Contents/Info.plist");
    let plist_str = plist_path.to_string_lossy();

    let output = runner::run_capture(
        "/usr/libexec/PlistBuddy",
        &["-c", "Print CFBundleShortVersionString", &plist_str],
    )?;

    let version = output.trim().to_string();
    if version.is_empty() {
        return Err(DmgrError::VersionExtractFailed);
    }

    output::info(&format!("App version: {}", version));
    Ok(version)
}

fn create_dmg(
    app_path: &Path,
    dmg_path: &Path,
    app_name: &str,
    config: &Config,
    dry_run: bool,
) -> Result<()> {
    if dmg_path.exists() {
        output::info(&format!("Removing existing DMG: {}", dmg_path.display()));
        if !dry_run {
            std::fs::remove_file(dmg_path)?;
        }
    }

    let app_str = app_path.to_string_lossy().to_string();
    let dmg_str = dmg_path.to_string_lossy().to_string();

    let volume_name = config.dmg.volume_name.as_deref().unwrap_or(app_name);

    let icon_size = config.dmg.icon_size.unwrap_or(100).to_string();
    let window_width = config.dmg.window_width.unwrap_or(600).to_string();
    let window_height = config.dmg.window_height.unwrap_or(400).to_string();
    let app_x = config.dmg.app_icon_x.unwrap_or(150).to_string();
    let app_y = config.dmg.app_icon_y.unwrap_or(200).to_string();
    let apps_x = config.dmg.applications_x.unwrap_or(450).to_string();
    let apps_y = config.dmg.applications_y.unwrap_or(200).to_string();
    let app_icon_name = format!("{}.app", app_name);

    let mut args: Vec<&str> = vec![
        "--volname",
        volume_name,
        "--icon-size",
        &icon_size,
        "--window-size",
        &window_width,
        &window_height,
        "--icon",
        &app_icon_name,
        &app_x,
        &app_y,
        "--icon",
        "Applications",
        &apps_x,
        &apps_y,
        "--app-drop-link",
        &apps_x,
        &apps_y,
    ];

    // Background image
    let bg_owned: String;
    if let Some(bg) = &config.dmg.background {
        bg_owned = bg.clone();
        args.push("--background");
        args.push(&bg_owned);
    }

    args.push(&dmg_str);
    args.push(&app_str);

    runner::run("create-dmg", &args, dry_run)?;
    Ok(())
}

fn codesign_dmg(dmg_path: &Path, identity: &str, dry_run: bool) -> Result<()> {
    let dmg_str = dmg_path.to_string_lossy();

    let args = vec!["--force", "--timestamp", "--sign", identity, &dmg_str];

    runner::run("codesign", &args, dry_run)?;
    output::success("DMG signed");
    Ok(())
}

fn notarize_dmg(dmg_path: &Path, keychain_profile: &str, dry_run: bool) -> Result<()> {
    let dmg_str = dmg_path.to_string_lossy();

    let args = vec![
        "notarytool",
        "submit",
        &dmg_str,
        "--keychain-profile",
        keychain_profile,
        "--wait",
    ];

    runner::run("xcrun", &args, dry_run)?;
    output::success("Notarization completed");
    Ok(())
}

fn staple_dmg(dmg_path: &Path, dry_run: bool) -> Result<()> {
    let dmg_str = dmg_path.to_string_lossy();
    let args = vec!["stapler", "staple", &dmg_str];

    runner::run("xcrun", &args, dry_run)?;
    output::success("Notarization ticket stapled");
    Ok(())
}

fn verify_notarization(dmg_path: &Path, dry_run: bool) -> Result<()> {
    let dmg_str = dmg_path.to_string_lossy();
    let args = vec![
        "-a",
        "-t",
        "open",
        "--context",
        "context:primary-signature",
        "-v",
        &dmg_str,
    ];

    runner::run("spctl", &args, dry_run)?;
    output::success("Gatekeeper approval verified");
    Ok(())
}
