use crate::config::Config;
use crate::error::{DmgrError, Result};
use crate::output;
use crate::runner;
use std::io::{self, Write};

pub fn run(
    list: bool,
    name: Option<String>,
    team_id: Option<String>,
    create_keychain: Option<String>,
) -> Result<()> {
    if list {
        return list_identities();
    }

    if let Some(profile_name) = create_keychain {
        return create_keychain_profile(&profile_name);
    }

    if name.is_some() || team_id.is_some() {
        return save_config(name, team_id);
    }

    Err(DmgrError::NoAction)
}

fn list_identities() -> Result<()> {
    output::step("Available signing identities");

    let output = runner::run_capture("security", &["find-identity", "-v", "-p", "codesigning"])?;

    let mut found = false;
    for line in output.lines() {
        if line.contains("Developer ID Application:") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start < end {
                        let identity = &line[start + 1..end];
                        output::success(identity);
                        found = true;
                    }
                }
            }
        }
    }

    if !found {
        output::warning("No Developer ID signing identities found");
        output::info("Install a certificate from Apple Developer Portal");
        return Err(DmgrError::NoIdentities);
    }

    Ok(())
}

fn save_config(name: Option<String>, team_id: Option<String>) -> Result<()> {
    let mut config = Config::load_global().unwrap_or_default();

    if let Some(n) = &name {
        config.signing.identity = Some(n.clone());
        output::success(&format!("Set signing identity: {}", n));
    }

    if let Some(t) = &team_id {
        config.signing.team_id = Some(t.clone());
        output::success(&format!("Set team ID: {}", t));
    }

    config.save_global()?;

    let path = Config::global_config_path()?;
    output::info(&format!("Saved to: {}", path.display()));

    Ok(())
}

fn profile_exists(profile_name: &str) -> bool {
    let result = std::process::Command::new("xcrun")
        .args([
            "notarytool",
            "info",
            "--keychain-profile",
            profile_name,
            "00000000-0000-0000-0000-000000000000",
        ])
        .output();

    match result {
        Ok(output) => {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            combined.contains("Submission does not exist")
                || combined.contains("does not belong to your team")
        }
        Err(_) => false,
    }
}

fn create_keychain_profile(profile_name: &str) -> Result<()> {
    if profile_exists(profile_name) {
        output::success(&format!(
            "Keychain profile '{}' already exists",
            profile_name
        ));

        let mut config = Config::load_global().unwrap_or_default();
        config.notarization.keychain_profile = Some(profile_name.to_string());
        config.save_global()?;

        output::success("Profile name saved to config");
        return Ok(());
    }

    output::step(&format!(
        "Creating notarization keychain profile: {}",
        profile_name
    ));
    println!();

    print!("Apple ID: ");
    io::stdout().flush()?;
    let mut apple_id = String::new();
    io::stdin().read_line(&mut apple_id)?;
    let apple_id = apple_id.trim();

    print!("App-specific password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    print!("Team ID: ");
    io::stdout().flush()?;
    let mut team_id = String::new();
    io::stdin().read_line(&mut team_id)?;
    let team_id = team_id.trim();

    println!();
    output::step("Running xcrun notarytool store-credentials");

    let args = vec![
        "notarytool",
        "store-credentials",
        profile_name,
        "--apple-id",
        apple_id,
        "--password",
        password,
        "--team-id",
        team_id,
    ];

    runner::run("xcrun", &args, false)?;

    output::success(&format!("Keychain profile '{}' created", profile_name));

    let mut config = Config::load_global().unwrap_or_default();
    config.notarization.keychain_profile = Some(profile_name.to_string());
    config.signing.team_id = Some(team_id.to_string());
    config.save_global()?;

    output::success("Profile name saved to config");

    Ok(())
}
