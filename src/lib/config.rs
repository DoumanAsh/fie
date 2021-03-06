//! Configuration module
use serde_derive::{Serialize, Deserialize};

///Describes which social platforms are enabled
///
///By default, if all platforms are not specified, then all are enabled.
///Otherwise, at least one is specified, each platform is assumed to be disabled
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
#[serde(default)]
pub struct Platforms {
    ///Whether Twitter is enabled
    #[serde(default)]
    pub twitter: bool,
    ///Whether Gab is enabled
    #[serde(default)]
    pub gab: bool,
    ///Whether Mastodon is enabled
    #[serde(default)]
    pub mastodon: bool,
    ///Whether Minds is enabled
    #[serde(default)]
    pub minds: bool,
}

// If the whole section on Platforms is missing then we assume
// that all platforms are used.
// If section is present though, missing field means that user doesn't want platform.
impl Default for Platforms {
    fn default() -> Self {
        Platforms {
            mastodon: true,
            gab: true,
            twitter: true,
            minds: true
        }
    }
}

///Pair of key and secret
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Token {
    ///Key
    pub key: String,
    ///Secret
    pub secret: String,
}
/// Twitter configuration
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Twitter {
    #[serde(default)]
    ///Consumer tokens, belongs to app.
    pub consumer: Token,
    #[serde(default)]
    ///Access tokens, granted per user.
    pub access: Token,
}

/// Gab configuration.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Gab {
    ///API's access token.
    ///
    ///Available through creating app on developer page
    #[serde(default)]
    pub access_token: String,
}

/// Mastodon configuration.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Mastodon {
    ///Hostname to connect
    #[serde(default)]
    pub host: String,
    ///API's access token.
    ///
    ///Available through creating app on developer page
    #[serde(default)]
    pub access_token: String,
}

/// Minds configuration.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Minds {
    ///Username for authorization
    #[serde(default)]
    pub username: String,
    ///Password for authorization
    #[serde(default)]
    pub password: String,
}

fn default_timeout() -> u64 {
    5
}

/// Fie's settings
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    #[serde(default = "default_timeout")]
    /// Amount of seconds to wait for all HTTP responses
    ///
    /// By default is 5.
    pub timeout: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { timeout: 5 }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
///Social media's API information
pub struct ApiConfig {
    ///Gab information
    pub gab: Gab,
    ///Twitter information
    pub twitter: Twitter,
    ///Mastodon information
    pub mastodon: Mastodon,
    ///Minds information
    pub minds: Minds,
}

///Fie's configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    ///Enable/disable switches for social medias
    #[serde(default)]
    pub platforms: Platforms,
    ///Social media's API information
    pub api: ApiConfig,
    ///Fie settings
    #[serde(default)]
    pub settings: Settings,
}
