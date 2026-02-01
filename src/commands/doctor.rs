use crate::config::Config;
use crate::error::{DmgrError, Result};
use crate::output;
use crate::runner;

pub fn run() -> Result<()> {
    output::header("dmgr doctor");
    println!();

    let mut all_ok = true;

    output::step("Checking required tools");
    all_ok &= check_tool("xcodebuild", true);
    all_ok &= check_tool("codesign", true);
    all_ok &= check_create_dmg();
    all_ok &= check_tool("xcrun", true);
    println!();

    output::step("Checking optional tools (Sparkle)");
    check_tool("generate_appcast", false);
    check_tool("sign_update", false);
    println!();

    output::step("Checking optional tools (Distribution)");
    check_gh_cli();
    check_aws_cli();
    println!();

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
                if verify_keychain_profile(profile) {
                    output::success(&format!("  Profile '{}' is valid", profile));
                } else {
                    output::warning(&format!("  Profile '{}' may be invalid", profile));
                }
            } else {
                output::info("No notarization profile configured");
                let profiles = find_keychain_profiles();
                if !profiles.is_empty() {
                    output::info("Found existing keychain profiles:");
                    for p in &profiles {
                        output::info(&format!("  - {}", p));
                    }
                    output::info("Configure with: dmgr profile --create-keychain <name>");
                } else {
                    output::info("Run: dmgr profile --create-keychain <name>");
                }
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

fn check_create_dmg() -> bool {
    match which::which("create-dmg") {
        Ok(path) => {
            let path_str = path.to_string_lossy();
            if path_str.contains("node") || path_str.contains("npm") {
                output::tool_status("create-dmg", true, path.to_str());
                output::warning("  Found npm version of create-dmg (incompatible)");
                output::info("  Run: npm uninstall -g create-dmg && brew install create-dmg");
                false
            } else {
                output::tool_status("create-dmg", true, path.to_str());
                true
            }
        }
        Err(_) => {
            output::tool_status("create-dmg", false, None);
            output::info("  Install with: brew install create-dmg");
            false
        }
    }
}

fn get_signing_identities() -> Result<Vec<String>> {
    let output = runner::run_capture("security", &["find-identity", "-v", "-p", "codesigning"])?;

    let mut identities = Vec::new();
    for line in output.lines() {
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

fn check_gh_cli() {
    match which::which("gh") {
        Ok(path) => {
            output::tool_status("gh", true, path.to_str());
            match runner::run_capture("gh", &["auth", "status"]) {
                Ok(_) => output::success("  authenticated"),
                Err(_) => {
                    output::warning("  not authenticated");
                    output::info("  Run: gh auth login");
                }
            }
        }
        Err(_) => {
            output::tool_status("gh", false, None);
            output::info("  Install with: brew install gh");
        }
    }
}

fn check_aws_cli() {
    match which::which("aws") {
        Ok(path) => {
            output::tool_status("aws", true, path.to_str());
            match runner::run_capture("aws", &["sts", "get-caller-identity"]) {
                Ok(_) => output::success("  configured"),
                Err(_) => {
                    output::warning("  not configured");
                    output::info("  Run: aws configure");
                }
            }
        }
        Err(_) => {
            output::tool_status("aws", false, None);
            output::info("  Install with: brew install awscli");
        }
    }
}

fn verify_keychain_profile(profile: &str) -> bool {
    let result = std::process::Command::new("xcrun")
        .args([
            "notarytool",
            "info",
            "--keychain-profile",
            profile,
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
                || (!combined.contains("credentials")
                    && !combined.contains("No credentials")
                    && !combined.contains("Could not find credentials"))
        }
        Err(_) => false,
    }
}

fn find_keychain_profiles() -> Vec<String> {
    let result = std::process::Command::new("security")
        .args(["dump-keychain"])
        .output();

    let Ok(output) = result else {
        return Vec::new();
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut profiles = Vec::new();
    let mut current_service: Option<String> = None;
    let mut is_notarytool = false;

    for line in stdout.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("\"svce\"") || trimmed.starts_with("0x00000007") {
            if let Some(start) = trimmed.find("=\"") {
                if let Some(end) = trimmed.rfind('"') {
                    let service = &trimmed[start + 2..end];
                    current_service = Some(service.to_string());
                }
            }
        }

        if trimmed.contains("notarytool") || trimmed.contains("notary") {
            is_notarytool = true;
        }

        if trimmed.contains("@") && trimmed.contains("apple") {
            is_notarytool = true;
        }

        if trimmed.starts_with("keychain:") || trimmed.is_empty() {
            if is_notarytool {
                if let Some(ref svc) = current_service {
                    if !svc.is_empty() && !profiles.contains(svc) {
                        if verify_keychain_profile(svc) {
                            profiles.push(svc.clone());
                        }
                    }
                }
            }
            current_service = None;
            is_notarytool = false;
        }
    }

    profiles
}
