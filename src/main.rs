mod appcast;
mod changelog;
mod commands;
mod config;
mod error;
mod menu;
mod output;
mod runner;
mod targets;

use clap::{Parser, Subcommand};
use commands::{archive, distribute, doctor, profile};
use error::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dmgr")]
#[command(about = "macOS app distribution manager - archive, sign, notarize, and distribute")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check system for required tools and configuration
    Doctor,

    /// Manage signing identity and notarization profiles
    Profile {
        /// List available signing identities
        #[arg(long)]
        list: bool,

        /// Set signing identity name (e.g., "Developer ID Application: Name (TEAM_ID)")
        #[arg(long)]
        name: Option<String>,

        /// Set team ID
        #[arg(long)]
        team_id: Option<String>,

        /// Create a notarization keychain profile
        #[arg(long, value_name = "PROFILE_NAME")]
        create_keychain: Option<String>,
    },

    /// Build, sign, and package app for distribution
    Archive {
        /// Xcode scheme to build
        #[arg(long)]
        scheme: String,

        /// Build configuration (Debug/Release)
        #[arg(long, default_value = "Release")]
        config: String,

        /// Output directory for DMG
        #[arg(long, short)]
        output: Option<String>,

        /// Signing identity (overrides config)
        #[arg(long)]
        identity: Option<String>,

        /// Notarize the DMG after creation
        #[arg(long)]
        notarize: bool,

        /// Show commands without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Distribute a DMG to GitHub releases, S3, and generate appcast
    Distribute {
        /// Path to the DMG file
        dmg_path: PathBuf,

        /// Path to changelog file
        #[arg(long, value_name = "PATH")]
        changelog: Option<PathBuf>,

        /// Path to existing appcast.xml to merge with
        #[arg(long, value_name = "PATH")]
        appcast: Option<PathBuf>,

        /// Output path for appcast.xml
        #[arg(long, value_name = "PATH")]
        output_appcast: Option<PathBuf>,

        /// Enable GitHub release
        #[arg(long)]
        github: bool,

        /// GitHub repository (owner/repo)
        #[arg(long, value_name = "OWNER/REPO")]
        github_repo: Option<String>,

        /// Enable S3 upload
        #[arg(long)]
        s3: bool,

        /// S3 bucket name
        #[arg(long, value_name = "BUCKET")]
        s3_bucket: Option<String>,

        /// S3 key prefix
        #[arg(long, value_name = "PREFIX")]
        s3_prefix: Option<String>,

        /// Override download URL in appcast
        #[arg(long, value_name = "URL")]
        download_url: Option<String>,

        /// Show commands without executing
        #[arg(long)]
        dry_run: bool,

        /// Skip changelog requirement
        #[arg(long)]
        skip_changelog: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result: Result<()> = match cli.command {
        None => menu::show(),
        Some(Commands::Doctor) => doctor::run(),
        Some(Commands::Profile {
            list,
            name,
            team_id,
            create_keychain,
        }) => profile::run(list, name, team_id, create_keychain),
        Some(Commands::Archive {
            scheme,
            config,
            output,
            identity,
            notarize,
            dry_run,
        }) => archive::run(scheme, config, output, identity, notarize, dry_run),
        Some(Commands::Distribute {
            dmg_path,
            changelog,
            appcast,
            output_appcast,
            github,
            github_repo,
            s3,
            s3_bucket,
            s3_prefix,
            download_url,
            dry_run,
            skip_changelog,
        }) => distribute::run(distribute::DistributeOptions {
            dmg_path,
            changelog_path: changelog,
            appcast_path: appcast,
            output_appcast_path: output_appcast,
            github_enabled: github,
            github_repo,
            s3_enabled: s3,
            s3_bucket,
            s3_prefix,
            download_url_override: download_url,
            dry_run,
            skip_changelog,
        }),
    };

    if let Err(e) = result {
        output::error(&e.to_string());
        std::process::exit(1);
    }
}
