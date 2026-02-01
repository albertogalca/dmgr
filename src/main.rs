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
use commands::{archive, distribute, doctor, profile, release};
use error::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dmgr")]
#[command(about = "macOS app distribution manager - archive, sign, notarize, and distribute")]
#[command(version)]
struct Cli {
    #[arg(long, short, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Doctor,
    Profile {
        #[arg(long)]
        list: bool,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        team_id: Option<String>,

        #[arg(long, value_name = "PROFILE_NAME")]
        create_keychain: Option<String>,
    },

    Archive {
        #[arg(long)]
        scheme: String,

        #[arg(long, default_value = "Release")]
        config: String,

        #[arg(long, short)]
        output: Option<String>,

        #[arg(long)]
        identity: Option<String>,

        #[arg(long)]
        notarize: bool,

        #[arg(long)]
        dry_run: bool,
    },

    Distribute {
        dmg_path: PathBuf,

        #[arg(long, value_name = "PATH")]
        changelog: Option<PathBuf>,

        #[arg(long, value_name = "PATH")]
        appcast: Option<PathBuf>,

        #[arg(long, value_name = "PATH")]
        output_appcast: Option<PathBuf>,

        #[arg(long)]
        github: bool,

        #[arg(long, value_name = "OWNER/REPO")]
        github_repo: Option<String>,

        #[arg(long)]
        s3: bool,

        #[arg(long, value_name = "BUCKET")]
        s3_bucket: Option<String>,

        #[arg(long, value_name = "PREFIX")]
        s3_prefix: Option<String>,

        #[arg(long, value_name = "URL")]
        download_url: Option<String>,

        #[arg(long)]
        dry_run: bool,

        #[arg(long)]
        skip_changelog: bool,
    },

    Release {
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

        /// Path to changelog file
        #[arg(long, value_name = "PATH")]
        changelog: Option<PathBuf>,

        /// Skip changelog requirement
        #[arg(long)]
        skip_changelog: bool,

        /// Show commands without executing
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Set quiet mode before running any commands
    output::set_quiet(cli.quiet);

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
            // CLI: if flag passed, use it; otherwise fall back to config
            github_enabled: if github { Some(true) } else { None },
            github_repo,
            s3_enabled: if s3 { Some(true) } else { None },
            s3_bucket,
            s3_prefix,
            download_url_override: download_url,
            dry_run,
            skip_changelog,
        }),
        Some(Commands::Release {
            scheme,
            config,
            output,
            identity,
            github,
            github_repo,
            s3,
            s3_bucket,
            s3_prefix,
            changelog,
            skip_changelog,
            dry_run,
        }) => release::run(release::ReleaseOptions {
            scheme,
            config,
            output_dir: output,
            identity,
            github_enabled: if github { Some(true) } else { None },
            github_repo,
            s3_enabled: if s3 { Some(true) } else { None },
            s3_bucket,
            s3_prefix,
            changelog_path: changelog,
            skip_changelog,
            dry_run,
        }),
    };

    if let Err(e) = result {
        output::error(&e.to_string());
        std::process::exit(1);
    }
}
