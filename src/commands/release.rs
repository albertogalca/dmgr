use crate::commands::{archive, distribute};
use crate::config::Config;
use crate::error::{DmgrError, Result};
use crate::output;
use std::path::PathBuf;

pub struct ReleaseOptions {
    pub scheme: String,
    pub config: String,
    pub output_dir: Option<String>,
    pub identity: Option<String>,
    pub github_enabled: Option<bool>,
    pub github_repo: Option<String>,
    pub s3_enabled: Option<bool>,
    pub s3_bucket: Option<String>,
    pub s3_prefix: Option<String>,
    pub changelog_path: Option<PathBuf>,
    pub skip_changelog: bool,
    pub dry_run: bool,
}

pub fn run(opts: ReleaseOptions) -> Result<()> {
    output::header(&format!("Releasing {}", opts.scheme));
    println!();

    let config = Config::load()?;

    let output_dir = opts
        .output_dir
        .clone()
        .unwrap_or_else(|| ".".to_string());

    output::step("Phase 1: Archive");
    println!();

    archive::run(
        opts.scheme.clone(),
        opts.config.clone(),
        Some(output_dir.clone()),
        opts.identity.clone(),
        true, // Always notarize for release
        opts.dry_run,
    )?;

    println!();

    output::step("Phase 2: Distribute");
    println!();

    let dmg_path = if opts.dry_run {
        PathBuf::from(format!("{}/{}-1.0.0.dmg", output_dir, opts.scheme))
    } else {
        find_latest_dmg(&output_dir, &opts.scheme)?
    };

    output::info(&format!("Found DMG: {}", dmg_path.display()));
    println!();

    let github_enabled = opts
        .github_enabled
        .unwrap_or_else(|| config.distribution.github.enabled.unwrap_or(false));
    let s3_enabled = opts
        .s3_enabled
        .unwrap_or_else(|| config.distribution.s3.enabled.unwrap_or(false));

    if !github_enabled && !s3_enabled {
        return Err(DmgrError::NoDistributionTarget);
    }

    distribute::run(distribute::DistributeOptions {
        dmg_path,
        changelog_path: opts.changelog_path,
        appcast_path: None,
        output_appcast_path: None,
        github_enabled: opts.github_enabled,
        github_repo: opts.github_repo,
        s3_enabled: opts.s3_enabled,
        s3_bucket: opts.s3_bucket,
        s3_prefix: opts.s3_prefix,
        download_url_override: None,
        dry_run: opts.dry_run,
        skip_changelog: opts.skip_changelog,
    })?;

    println!();
    output::success(&format!("Release of {} completed", opts.scheme));

    Ok(())
}

fn find_latest_dmg(output_dir: &str, scheme: &str) -> Result<PathBuf> {
    let dir = PathBuf::from(output_dir);

    if !dir.exists() {
        return Err(DmgrError::DmgNotFound(format!(
            "Output directory does not exist: {}",
            output_dir
        )));
    }

    let prefix = format!("{}-", scheme);
    let mut best_match: Option<(PathBuf, std::time::SystemTime)> = None;

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(&prefix) && name.ends_with(".dmg") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if best_match
                            .as_ref()
                            .map(|(_, t)| modified > *t)
                            .unwrap_or(true)
                        {
                            best_match = Some((path, modified));
                        }
                    }
                }
            }
        }
    }

    best_match
        .map(|(path, _)| path)
        .ok_or_else(|| DmgrError::DmgNotFound(format!("No DMG found for {} in {}", scheme, output_dir)))
}
