//! Configuration module
use serde_derive::{Deserialize};

///Describes which social platforms are enabled
///
///By default, if all platforms are not specified, then all are enabled.
///Otherwise, at least one is specified, each platform is assumed to be disabled
#[derive(Deserialize, Copy, Clone, Debug)]
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
        }
    }
}

///Pair of key and secret
#[derive(Deserialize, Debug)]
pub struct Token {
    ///Key
    pub key: String,
    ///Secret
    pub secret: String,
}
/// Twitter configuration
#[derive(Deserialize, Debug)]
pub struct Twitter {
    ///Consumer tokens, belongs to app.
    pub consumer: Token,
    ///Access tokens, granted per user.
    pub access: Token,
}

/// Gab configuration.
#[derive(Deserialize, Debug)]
pub struct Gab {
    ///Username for authorization
    #[serde(default)]
    pub username: String,
    ///Password for authorization
    #[serde(default)]
    pub password: String,
}

/// Mastodon configuration.
#[derive(Deserialize, Debug)]
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

fn default_timeout() -> u64 {
    5
}

/// Fie's settings
#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug)]
///Social media's API information
pub struct ApiConfig {
    ///Gab information
    pub gab: Gab,
    ///Twitter information
    pub twitter: Twitter,
    ///Mastodon information
    pub mastodon: Mastodon,
}

///Fie's configuration
#[derive(Deserialize, Debug)]
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
