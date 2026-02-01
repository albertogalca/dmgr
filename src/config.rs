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
    pub identity: Option<String>,
    pub team_id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NotarizationConfig {
    pub keychain_profile: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DmgConfig {
    pub background: Option<String>,
    pub volume_name: Option<String>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub icon_size: Option<u32>,
    pub app_icon_x: Option<u32>,
    pub app_icon_y: Option<u32>,
    pub applications_x: Option<u32>,
    pub applications_y: Option<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SparkleConfig {
    pub private_key: Option<String>,
    pub appcast_url: Option<String>,
    pub appcast_output: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DistributionConfig {
    pub changelog: Option<String>,
    #[serde(default)]
    pub github: GitHubConfig,
    #[serde(default)]
    pub s3: S3Config,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub enabled: Option<bool>,
    pub repo: Option<String>,
    pub tag_prefix: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct S3Config {
    pub enabled: Option<bool>,
    pub bucket: Option<String>,
    pub prefix: Option<String>,
    pub region: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let global = Self::load_global().unwrap_or_default();
        let project = Self::load_project().unwrap_or_default();
        Ok(global.merge(project))
    }

    pub fn load_global() -> Result<Self> {
        let path = Self::global_config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn load_project() -> Result<Self> {
        let path = PathBuf::from(".dmgr.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn save_global(&self) -> Result<()> {
        let path = Self::global_config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    pub fn global_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(home.join(".config").join("dmgr").join("config.toml"))
    }

    fn merge(mut self, other: Self) -> Self {
        if other.signing.identity.is_some() {
            self.signing.identity = other.signing.identity;
        }
        if other.signing.team_id.is_some() {
            self.signing.team_id = other.signing.team_id;
        }

        if other.notarization.keychain_profile.is_some() {
            self.notarization.keychain_profile = other.notarization.keychain_profile;
        }

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

        if other.sparkle.private_key.is_some() {
            self.sparkle.private_key = other.sparkle.private_key;
        }
        if other.sparkle.appcast_url.is_some() {
            self.sparkle.appcast_url = other.sparkle.appcast_url;
        }
        if other.sparkle.appcast_output.is_some() {
            self.sparkle.appcast_output = other.sparkle.appcast_output;
        }

        if other.distribution.changelog.is_some() {
            self.distribution.changelog = other.distribution.changelog;
        }

        if other.distribution.github.enabled.is_some() {
            self.distribution.github.enabled = other.distribution.github.enabled;
        }
        if other.distribution.github.repo.is_some() {
            self.distribution.github.repo = other.distribution.github.repo;
        }
        if other.distribution.github.tag_prefix.is_some() {
            self.distribution.github.tag_prefix = other.distribution.github.tag_prefix;
        }

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
