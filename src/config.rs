use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Could not determine config directory")]
    NoConfigDir,
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub signing: SigningConfig,

    #[serde(default)]
    pub notarization: NotarizationConfig,

    #[serde(default)]
    pub dmg: DmgConfig,

    #[serde(default)]
    pub sparkle: SparkleConfig,

    #[serde(default)]
    pub distribution: DistributionConfig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SigningConfig {
    /// Full signing identity name (e.g., "Developer ID Application: Name (TEAM_ID)")
    pub identity: Option<String>,

    /// Team ID
    pub team_id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NotarizationConfig {
    /// Keychain profile name for notarytool
    pub keychain_profile: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DmgConfig {
    /// Path to background image
    pub background: Option<String>,

    /// Volume name (defaults to app name)
    pub volume_name: Option<String>,

    /// Window width
    pub window_width: Option<u32>,

    /// Window height
    pub window_height: Option<u32>,

    /// Icon size
    pub icon_size: Option<u32>,

    /// App icon X position
    pub app_icon_x: Option<u32>,

    /// App icon Y position
    pub app_icon_y: Option<u32>,

    /// Applications symlink X position
    pub applications_x: Option<u32>,

    /// Applications symlink Y position
    pub applications_y: Option<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SparkleConfig {
    /// Path to EdDSA private key for Sparkle signing
    pub private_key: Option<String>,

    /// URL where appcast.xml will be hosted
    pub appcast_url: Option<String>,

    /// Path to output appcast.xml
    pub appcast_output: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DistributionConfig {
    /// Path to changelog file
    pub changelog: Option<String>,

    /// GitHub release settings
    #[serde(default)]
    pub github: GitHubConfig,

    /// S3 upload settings
    #[serde(default)]
    pub s3: S3Config,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// Enable GitHub releases
    pub enabled: Option<bool>,

    /// Repository in owner/repo format
    pub repo: Option<String>,

    /// Tag prefix (e.g., "v" for v1.0.0)
    pub tag_prefix: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct S3Config {
    /// Enable S3 uploads
    pub enabled: Option<bool>,

    /// S3 bucket name
    pub bucket: Option<String>,

    /// Key prefix within the bucket
    pub prefix: Option<String>,

    /// AWS region
    pub region: Option<String>,
}

impl Config {
    /// Load configuration, merging global and project configs
    /// Project config (.dmgr.toml) overrides global config
    pub fn load() -> Result<Self> {
        let global = Self::load_global().unwrap_or_default();
        let project = Self::load_project().unwrap_or_default();
        Ok(global.merge(project))
    }

    /// Load global config from ~/.config/dmgr/config.toml
    pub fn load_global() -> Result<Self> {
        let path = Self::global_config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path)?;
        Ok(toml::from_str(&contents)?)
    }

    /// Load project config from .dmgr.toml in current directory
    pub fn load_project() -> Result<Self> {
        let path = PathBuf::from(".dmgr.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path)?;
        Ok(toml::from_str(&contents)?)
    }

    /// Save config to global config file
    pub fn save_global(&self) -> Result<()> {
        let path = Self::global_config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    /// Get the path to the global config file
    pub fn global_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(config_dir.join("dmgr").join("config.toml"))
    }

    /// Merge another config into this one (other takes precedence)
    fn merge(mut self, other: Self) -> Self {
        // Signing
        if other.signing.identity.is_some() {
            self.signing.identity = other.signing.identity;
        }
        if other.signing.team_id.is_some() {
            self.signing.team_id = other.signing.team_id;
        }

        // Notarization
        if other.notarization.keychain_profile.is_some() {
            self.notarization.keychain_profile = other.notarization.keychain_profile;
        }

        // DMG
        if other.dmg.background.is_some() {
            self.dmg.background = other.dmg.background;
        }
        if other.dmg.volume_name.is_some() {
            self.dmg.volume_name = other.dmg.volume_name;
        }
        if other.dmg.window_width.is_some() {
            self.dmg.window_width = other.dmg.window_width;
        }
        if other.dmg.window_height.is_some() {
            self.dmg.window_height = other.dmg.window_height;
        }
        if other.dmg.icon_size.is_some() {
            self.dmg.icon_size = other.dmg.icon_size;
        }
        if other.dmg.app_icon_x.is_some() {
            self.dmg.app_icon_x = other.dmg.app_icon_x;
        }
        if other.dmg.app_icon_y.is_some() {
            self.dmg.app_icon_y = other.dmg.app_icon_y;
        }
        if other.dmg.applications_x.is_some() {
            self.dmg.applications_x = other.dmg.applications_x;
        }
        if other.dmg.applications_y.is_some() {
            self.dmg.applications_y = other.dmg.applications_y;
        }

        // Sparkle
        if other.sparkle.private_key.is_some() {
            self.sparkle.private_key = other.sparkle.private_key;
        }
        if other.sparkle.appcast_url.is_some() {
            self.sparkle.appcast_url = other.sparkle.appcast_url;
        }
        if other.sparkle.appcast_output.is_some() {
            self.sparkle.appcast_output = other.sparkle.appcast_output;
        }

        // Distribution
        if other.distribution.changelog.is_some() {
            self.distribution.changelog = other.distribution.changelog;
        }

        // Distribution - GitHub
        if other.distribution.github.enabled.is_some() {
            self.distribution.github.enabled = other.distribution.github.enabled;
        }
        if other.distribution.github.repo.is_some() {
            self.distribution.github.repo = other.distribution.github.repo;
        }
        if other.distribution.github.tag_prefix.is_some() {
            self.distribution.github.tag_prefix = other.distribution.github.tag_prefix;
        }

        // Distribution - S3
        if other.distribution.s3.enabled.is_some() {
            self.distribution.s3.enabled = other.distribution.s3.enabled;
        }
        if other.distribution.s3.bucket.is_some() {
            self.distribution.s3.bucket = other.distribution.s3.bucket;
        }
        if other.distribution.s3.prefix.is_some() {
            self.distribution.s3.prefix = other.distribution.s3.prefix;
        }
        if other.distribution.s3.region.is_some() {
            self.distribution.s3.region = other.distribution.s3.region;
        }

        self
    }
}
