use crate::commands::{archive, distribute, doctor, profile};
use crate::config::Config;
use crate::error::Result;
use crate::output;
use dialoguer::{Confirm, Input, Select};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

/// Get available Xcode schemes and configurations from current directory
fn get_xcode_info() -> Option<(Vec<String>, Vec<String>)> {
    print!("Detecting Xcode schemes... ");
    let _ = io::stdout().flush();

    let output = Command::new("xcodebuild")
        .arg("-list")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut schemes = Vec::new();
    let mut configs = Vec::new();
    let mut in_schemes = false;
    let mut in_configs = false;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed == "Schemes:" {
            in_schemes = true;
            in_configs = false;
            continue;
        } else if trimmed == "Build Configurations:" {
            in_configs = true;
            in_schemes = false;
            continue;
        } else if trimmed.ends_with(':') || trimmed.is_empty() {
            if !trimmed.is_empty() {
                in_schemes = false;
                in_configs = false;
            }
            continue;
        }

        if in_schemes && !trimmed.is_empty() {
            schemes.push(trimmed.to_string());
        } else if in_configs && !trimmed.is_empty() {
            configs.push(trimmed.to_string());
        }
    }

    if schemes.is_empty() {
        println!("none found");
        None
    } else {
        println!("found {} scheme(s)", schemes.len());
        Some((schemes, configs))
    }
}

/// Get available codesigning identities
fn get_signing_identities() -> Vec<String> {
    let output = Command::new("security")
        .args(["find-identity", "-v", "-p", "codesigning"])
        .output()
        .ok();

    let Some(output) = output else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut identities = Vec::new();

    for line in stdout.lines() {
        // Parse lines like: 1) HASH "Identity Name"
        if let Some(start) = line.find('"') {
            if let Some(end) = line.rfind('"') {
                if start < end {
                    let identity = line[start + 1..end].to_string();
                    identities.push(identity);
                }
            }
        }
    }

    identities
}

#[derive(Debug, Clone, Copy)]
enum MainCommand {
    Doctor,
    Profile,
    Archive,
    Distribute,
}

impl MainCommand {
    fn all() -> &'static [MainCommand] {
        &[
            MainCommand::Doctor,
            MainCommand::Profile,
            MainCommand::Archive,
            MainCommand::Distribute,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            MainCommand::Doctor => "Doctor",
            MainCommand::Profile => "Profile",
            MainCommand::Archive => "Archive",
            MainCommand::Distribute => "Distribute",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            MainCommand::Doctor => "Check system for required tools",
            MainCommand::Profile => "Manage signing identity and profiles",
            MainCommand::Archive => "Build, sign, and package app",
            MainCommand::Distribute => "Upload DMG and generate appcast",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ProfileAction {
    ListIdentities,
    SetIdentity,
    CreateKeychainProfile,
}

impl ProfileAction {
    fn all() -> &'static [ProfileAction] {
        &[
            ProfileAction::ListIdentities,
            ProfileAction::SetIdentity,
            ProfileAction::CreateKeychainProfile,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            ProfileAction::ListIdentities => "List signing identities",
            ProfileAction::SetIdentity => "Set signing identity",
            ProfileAction::CreateKeychainProfile => "Create keychain profile",
        }
    }
}

pub fn show() -> Result<()> {
    output::menu_header();

    loop {
        let items: Vec<String> = MainCommand::all()
            .iter()
            .enumerate()
            .map(|(i, cmd)| format!("{}. {}  - {}", i + 1, cmd.label(), cmd.description()))
            .collect();

        let selection = Select::new()
            .items(&items)
            .default(0)
            .with_prompt("Select command")
            .interact_opt()?;

        match selection {
            Some(idx) => {
                let cmd = MainCommand::all()[idx];
                if let Err(e) = execute_command(cmd) {
                    output::error(&e.to_string());
                }
                println!();

                // Ask if user wants to run another command
                let run_another = Confirm::new()
                    .with_prompt("Run another command?")
                    .default(false)
                    .interact_opt()?;

                match run_another {
                    Some(true) => {
                        println!();
                        continue;
                    }
                    _ => break,
                }
            }
            None => {
                // User pressed Escape or Ctrl+C
                break;
            }
        }
    }

    Ok(())
}

fn execute_command(cmd: MainCommand) -> Result<()> {
    match cmd {
        MainCommand::Doctor => doctor::run(),
        MainCommand::Profile => run_profile_menu(),
        MainCommand::Archive => run_archive_prompts(),
        MainCommand::Distribute => run_distribute_prompts(),
    }
}

fn run_profile_menu() -> Result<()> {
    let items: Vec<&str> = ProfileAction::all().iter().map(|a| a.label()).collect();

    let selection = Select::new()
        .items(&items)
        .default(0)
        .with_prompt("Profile action")
        .interact_opt()?;

    match selection {
        Some(0) => profile::run(true, None, None, None),
        Some(1) => {
            let name: String = Input::new()
                .with_prompt("Signing identity name")
                .interact_text()?;
            let team_id: String = Input::new()
                .with_prompt("Team ID (optional)")
                .allow_empty(true)
                .interact_text()?;
            let team = if team_id.is_empty() {
                None
            } else {
                Some(team_id)
            };
            profile::run(false, Some(name), team, None)
        }
        Some(2) => {
            let profile_name: String = Input::new()
                .with_prompt("Keychain profile name")
                .interact_text()?;
            profile::run(false, None, None, Some(profile_name))
        }
        _ => Ok(()),
    }
}

fn run_archive_prompts() -> Result<()> {
    // Load config first to check what's already configured
    let dmgr_config = Config::load().unwrap_or_default();

    let xcode_info = get_xcode_info();

    let scheme: String = if let Some((ref schemes, _)) = xcode_info {
        let idx = Select::new()
            .items(schemes)
            .default(0)
            .with_prompt("Xcode scheme")
            .interact()?;
        schemes[idx].clone()
    } else {
        Input::new()
            .with_prompt("Xcode scheme")
            .interact_text()?
    };

    let config: String = if let Some((_, ref configs)) = xcode_info {
        if configs.is_empty() {
            "Release".to_string()
        } else {
            let default_idx = configs.iter().position(|c| c == "Release").unwrap_or(0);
            let idx = Select::new()
                .items(configs)
                .default(default_idx)
                .with_prompt("Build configuration")
                .interact()?;
            configs[idx].clone()
        }
    } else {
        Input::new()
            .with_prompt("Build configuration")
            .default("Release".to_string())
            .interact_text()?
    };

    let output_dir: String = Input::new()
        .with_prompt("Output directory (optional)")
        .allow_empty(true)
        .interact_text()?;
    let output = if output_dir.is_empty() {
        None
    } else {
        Some(output_dir)
    };

    // Skip signing identity prompt if already configured
    let identity = if dmgr_config.signing.identity.is_some() {
        output::info(&format!(
            "Using signing identity from config: {}",
            dmgr_config.signing.identity.as_ref().unwrap()
        ));
        None // None means use config default
    } else {
        let identities = get_signing_identities();
        if identities.is_empty() {
            let identity_str: String = Input::new()
                .with_prompt("Signing identity")
                .allow_empty(true)
                .interact_text()?;
            if identity_str.is_empty() {
                None
            } else {
                Some(identity_str)
            }
        } else {
            // Find first Developer ID identity as default
            let default_idx = identities
                .iter()
                .position(|id| id.contains("Developer ID"))
                .unwrap_or(0);

            let idx = Select::new()
                .items(&identities)
                .default(default_idx)
                .with_prompt("Signing identity")
                .interact()?;

            Some(identities[idx].clone())
        }
    };

    let has_notarization_profile = dmgr_config.notarization.keychain_profile.is_some();

    let notarize = if has_notarization_profile {
        let profile_name = dmgr_config.notarization.keychain_profile.as_ref().unwrap();
        Confirm::new()
            .with_prompt(format!("Notarize after creation? (profile: {})", profile_name))
            .default(true)
            .interact()?
    } else {
        let want_notarize = Confirm::new()
            .with_prompt("Notarize after creation? (requires keychain profile)")
            .default(false)
            .interact()?;

        if want_notarize {
            output::warning("No keychain profile configured.");
            let create_now = Confirm::new()
                .with_prompt("Create one now?")
                .default(true)
                .interact()?;

            if create_now {
                let profile_name: String = Input::new()
                    .with_prompt("Keychain profile name")
                    .default("notarization".to_string())
                    .interact_text()?;
                profile::run(false, None, None, Some(profile_name))?;
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    let dry_run = Confirm::new()
        .with_prompt("Dry run (show commands only)?")
        .default(false)
        .interact()?;

    archive::run(scheme, config, output, identity, notarize, dry_run)
}

fn run_distribute_prompts() -> Result<()> {
    // Load config to pre-fill values
    let dmgr_config = Config::load().unwrap_or_default();

    let dmg_path_str: String = Input::new()
        .with_prompt("Path to DMG file")
        .interact_text()?;
    let dmg_path = PathBuf::from(dmg_path_str);

    // Pre-fill changelog from config
    let default_changelog = dmgr_config
        .distribution
        .changelog
        .as_deref()
        .unwrap_or("");
    let changelog_str: String = Input::new()
        .with_prompt("Changelog file path (optional)")
        .default(default_changelog.to_string())
        .allow_empty(true)
        .interact_text()?;
    let changelog = if changelog_str.is_empty() {
        None
    } else {
        Some(PathBuf::from(changelog_str))
    };

    let appcast_str: String = Input::new()
        .with_prompt("Existing appcast.xml path (optional)")
        .default(String::new())
        .allow_empty(true)
        .interact_text()?;
    let appcast = if appcast_str.is_empty() {
        None
    } else {
        Some(PathBuf::from(appcast_str))
    };

    // Pre-fill output appcast from config
    let default_appcast_output = dmgr_config
        .sparkle
        .appcast_output
        .as_deref()
        .unwrap_or("");
    let output_appcast_str: String = Input::new()
        .with_prompt("Output appcast.xml path (optional)")
        .default(default_appcast_output.to_string())
        .allow_empty(true)
        .interact_text()?;
    let output_appcast = if output_appcast_str.is_empty() {
        None
    } else {
        Some(PathBuf::from(output_appcast_str))
    };

    // Pre-fill GitHub settings from config
    let config_github_enabled = dmgr_config.distribution.github.enabled.unwrap_or(false);
    let config_github_repo = dmgr_config.distribution.github.repo.as_deref();

    let github = if config_github_repo.is_some() {
        Confirm::new()
            .with_prompt(format!(
                "Upload to GitHub release? (repo: {})",
                config_github_repo.unwrap()
            ))
            .default(config_github_enabled)
            .interact()?
    } else {
        Confirm::new()
            .with_prompt("Upload to GitHub release?")
            .default(false)
            .interact()?
    };

    let github_repo = if github {
        if let Some(repo) = config_github_repo {
            output::info(&format!("Using GitHub repo from config: {}", repo));
            None // None means use config default
        } else {
            let repo: String = Input::new()
                .with_prompt("GitHub repository (owner/repo)")
                .interact_text()?;
            Some(repo)
        }
    } else {
        None
    };

    // Pre-fill S3 settings from config
    let config_s3_enabled = dmgr_config.distribution.s3.enabled.unwrap_or(false);
    let config_s3_bucket = dmgr_config.distribution.s3.bucket.as_deref();
    let config_s3_prefix = dmgr_config.distribution.s3.prefix.as_deref();

    let s3 = if config_s3_bucket.is_some() {
        Confirm::new()
            .with_prompt(format!(
                "Upload to S3? (bucket: {})",
                config_s3_bucket.unwrap()
            ))
            .default(config_s3_enabled)
            .interact()?
    } else {
        Confirm::new()
            .with_prompt("Upload to S3?")
            .default(false)
            .interact()?
    };

    let (s3_bucket, s3_prefix) = if s3 {
        if config_s3_bucket.is_some() {
            output::info(&format!(
                "Using S3 bucket from config: {}",
                config_s3_bucket.unwrap()
            ));
            if let Some(prefix) = config_s3_prefix {
                output::info(&format!("Using S3 prefix from config: {}", prefix));
            }
            (None, None) // None means use config defaults
        } else {
            let bucket: String = Input::new()
                .with_prompt("S3 bucket name")
                .interact_text()?;
            let prefix: String = Input::new()
                .with_prompt("S3 key prefix (optional)")
                .allow_empty(true)
                .interact_text()?;
            (
                Some(bucket),
                if prefix.is_empty() { None } else { Some(prefix) },
            )
        }
    } else {
        (None, None)
    };

    let download_url_str: String = Input::new()
        .with_prompt("Override download URL (optional)")
        .default(String::new())
        .allow_empty(true)
        .interact_text()?;
    let download_url = if download_url_str.is_empty() {
        None
    } else {
        Some(download_url_str)
    };

    let skip_changelog = if changelog.is_none() {
        Confirm::new()
            .with_prompt("Skip changelog requirement?")
            .default(false)
            .interact()?
    } else {
        false
    };

    let dry_run = Confirm::new()
        .with_prompt("Dry run (show commands only)?")
        .default(false)
        .interact()?;

    distribute::run(distribute::DistributeOptions {
        dmg_path,
        changelog_path: changelog,
        appcast_path: appcast,
        output_appcast_path: output_appcast,
        github_enabled: Some(github), // Explicit user choice
        github_repo,
        s3_enabled: Some(s3), // Explicit user choice
        s3_bucket,
        s3_prefix,
        download_url_override: download_url,
        dry_run,
        skip_changelog,
    })
}
