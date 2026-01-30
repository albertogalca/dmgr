use thiserror::Error;

#[derive(Error, Debug)]
pub enum DmgrError {
    #[error("Missing required tools")]
    MissingTools,

    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),

    #[error("Runner error: {0}")]
    RunnerError(#[from] crate::runner::RunnerError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("No signing identity configured. Run: dmgr profile --list")]
    NoSigningIdentity,

    #[error("No notarization profile configured. Run: dmgr profile --create-keychain <name>")]
    NoNotarizationProfile,

    #[allow(dead_code)]
    #[error("App bundle not found at: {0}")]
    AppNotFound(String),

    #[error("Failed to extract version from Info.plist")]
    VersionExtractFailed,

    #[error("No signing identities found")]
    NoIdentities,

    #[error("No action specified. Use --list, --name/--team-id, or --create-keychain")]
    NoAction,
}

pub type Result<T> = std::result::Result<T, DmgrError>;
