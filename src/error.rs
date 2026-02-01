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

    #[error("DMG file not found: {0}")]
    DmgNotFound(String),

    #[error("Failed to mount DMG")]
    DmgMountFailed,

    #[error("No .app bundle found in DMG")]
    NoAppInDmg,

    #[error("Changelog not found: {0}")]
    ChangelogNotFound(String),

    #[error("No changelog entry for version {0}")]
    ChangelogEntryMissing(String),

    #[error("Changelog parse error: {0}")]
    ChangelogParseError(String),

    #[error("Appcast error: {0}")]
    AppcastError(String),

    #[error("GitHub error: {0}")]
    GitHubError(String),

    #[error("S3 error: {0}")]
    S3Error(String),

    #[error("No distribution target specified. Use --github or --s3")]
    NoDistributionTarget,

    #[error("GitHub repo not specified. Use --github-repo or set in .dmgr.toml")]
    NoGitHubRepo,

    #[error("S3 bucket not specified. Use --s3-bucket or set in .dmgr.toml")]
    NoS3Bucket,

    #[error("No download URL available")]
    NoDownloadUrl,

    #[error("Menu error: {0}")]
    MenuError(String),
}

impl From<dialoguer::Error> for DmgrError {
    fn from(err: dialoguer::Error) -> Self {
        DmgrError::MenuError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DmgrError>;
