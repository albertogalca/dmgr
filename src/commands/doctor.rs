use crate::config::Config;
use crate::error::{DmgrError, Result};
use crate::output;
use crate::runner;

pub fn run() -> Result<()> {
    output::header("dmgr doctor");
    println!();

    let mut all_ok = true;

    // Check required tools
    output::step("Checking required tools");
    all_ok &= check_tool("xcodebuild", true);
    all_ok &= check_tool("codesign", true);
    all_ok &= check_tool("create-dmg", true);
    all_ok &= check_tool("xcrun", true);
    println!();

    // Check optional tools (Sparkle)
    output::step("Checking optional tools (Sparkle)");
    check_tool("generate_appcast", false);
    check_tool("sign_update", false);
    println!();

    // Check signing identities
    output::step("Checking signing identities");
    match get_signing_identities() {
        Ok(identities) => {
            if identities.is_empty() {
                output::warning("No Developer ID signing identities found");
                output::info("Install a Developer ID certificate from Apple Developer Portal");
            } else {
                for identity in &identities {
                    output::success(identity);
                }
            }
        }
        Err(e) => {
            output::error(&format!("Failed to query keychain: {}", e));
            all_ok = false;
        }
    }
    println!();

    // Check configuration
    output::step("Checking configuration");
    match Config::load() {
        Ok(config) => {
            if let Some(identity) = &config.signing.identity {
                output::list_item("Signing identity", identity);
            } else {
                output::info("No signing identity configured (will use first available)");
            }

            if let Some(team_id) = &config.signing.team_id {
                output::list_item("Team ID", team_id);
            }

            if let Some(profile) = &config.notarization.keychain_profile {
                output::list_item("Notarization profile", profile);
            } else {
                output::info("No notarization profile configured");
                output::info("Run: dmgr profile --create-keychain <name>");
            }
        }
        Err(e) => {
            output::warning(&format!("Could not load config: {}", e));
        }
    }
    println!();

    if all_ok {
        output::success("All required tools are available");
        Ok(())
    } else {
        output::error("Some required tools are missing");
        Err(DmgrError::MissingTools)
    }
}

fn check_tool(name: &str, required: bool) -> bool {
    match which::which(name) {
        Ok(path) => {
            output::tool_status(name, true, path.to_str());
            true
        }
        Err(_) => {
            output::tool_status(name, false, None);
            !required
        }
    }
}

/// Query keychain for Developer ID signing identities
fn get_signing_identities() -> Result<Vec<String>> {
    let output = runner::run_capture("security", &["find-identity", "-v", "-p", "codesigning"])?;

    let mut identities = Vec::new();
    for line in output.lines() {
        // Lines look like: 1) HASH "Developer ID Application: Name (TEAM_ID)"
        if line.contains("Developer ID Application:") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start < end {
                        identities.push(line[start + 1..end].to_string());
                    }
                }
            }
        }
    }

    Ok(identities)
}
