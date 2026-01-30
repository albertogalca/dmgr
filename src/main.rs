mod commands;
mod config;
mod error;
mod output;
mod runner;

use clap::{Parser, Subcommand};
use commands::{archive, doctor, profile};
use error::Result;

#[derive(Parser)]
#[command(name = "dmgr")]
#[command(about = "macOS app distribution manager - archive, sign, notarize, and distribute")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
}

fn main() {
    let cli = Cli::parse();

    let result: Result<()> = match cli.command {
        Commands::Doctor => doctor::run(),
        Commands::Profile {
            list,
            name,
            team_id,
            create_keychain,
        } => profile::run(list, name, team_id, create_keychain),
        Commands::Archive {
            scheme,
            config,
            output,
            identity,
            notarize,
            dry_run,
        } => archive::run(scheme, config, output, identity, notarize, dry_run),
    };

    if let Err(e) = result {
        output::error(&e.to_string());
        std::process::exit(1);
    }
}
