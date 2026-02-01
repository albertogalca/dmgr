use crate::output;
use crate::runner;
use std::path::Path;

use super::UploadResult;

/// Create or update a GitHub release and upload the DMG
pub fn upload_release(
    repo: &str,
    tag: &str,
    dmg_path: &Path,
    release_notes: Option<&str>,
    dry_run: bool,
) -> Result<UploadResult, String> {
    let dmg_str = dmg_path.to_string_lossy();
    let dmg_name = dmg_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid DMG filename")?;

    // Check if release exists
    let release_exists = check_release_exists(repo, tag)?;

    if release_exists {
        output::info(&format!("Updating existing release: {}", tag));
        // Delete existing asset if present
        if let Err(e) = delete_release_asset(repo, tag, dmg_name, dry_run) {
            output::info(&format!("Note: {}", e));
        }
    } else {
        output::info(&format!("Creating new release: {}", tag));
        create_release(repo, tag, release_notes, dry_run)?;
    }

    // Upload asset
    upload_asset(repo, tag, &dmg_str, dry_run)?;

    // Get download URL
    let download_url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        repo, tag, dmg_name
    );

    output::success(&format!("Uploaded to GitHub: {}", download_url));

    Ok(UploadResult {
        download_url,
        metadata: Some(format!("Release: {}", tag)),
    })
}

/// Check if a release exists
fn check_release_exists(repo: &str, tag: &str) -> Result<bool, String> {
    let args = vec!["release", "view", tag, "--repo", repo];
    match runner::run_capture("gh", &args) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Create a new release
fn create_release(
    repo: &str,
    tag: &str,
    notes: Option<&str>,
    dry_run: bool,
) -> Result<(), String> {
    let title = format!("Release {}", tag);

    let mut args = vec![
        "release",
        "create",
        tag,
        "--repo",
        repo,
        "--title",
        &title,
    ];

    let notes_owned: String;
    if let Some(n) = notes {
        notes_owned = n.to_string();
        args.push("--notes");
        args.push(&notes_owned);
    } else {
        args.push("--generate-notes");
    }

    runner::run("gh", &args, dry_run).map_err(|e| format!("Failed to create release: {}", e))?;
    Ok(())
}

/// Delete an existing release asset
fn delete_release_asset(
    repo: &str,
    tag: &str,
    asset_name: &str,
    dry_run: bool,
) -> Result<(), String> {
    let args = vec![
        "release",
        "delete-asset",
        tag,
        asset_name,
        "--repo",
        repo,
        "--yes",
    ];

    runner::run("gh", &args, dry_run)
        .map_err(|e| format!("Failed to delete existing asset: {}", e))?;
    Ok(())
}

/// Upload an asset to a release
fn upload_asset(repo: &str, tag: &str, file_path: &str, dry_run: bool) -> Result<(), String> {
    let args = vec![
        "release",
        "upload",
        tag,
        file_path,
        "--repo",
        repo,
        "--clobber",
    ];

    runner::run("gh", &args, dry_run).map_err(|e| format!("Failed to upload asset: {}", e))?;
    Ok(())
}

/// Check if gh CLI is available and authenticated
#[allow(dead_code)]
pub fn check_available() -> Result<(), String> {
    // Check if gh is installed
    which::which("gh").map_err(|_| "gh CLI not found. Install with: brew install gh".to_string())?;

    // Check if authenticated
    let args = vec!["auth", "status"];
    runner::run_capture("gh", &args).map_err(|_| {
        "gh CLI not authenticated. Run: gh auth login".to_string()
    })?;

    Ok(())
}
