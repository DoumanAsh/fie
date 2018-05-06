//!Configuration module
extern crate toml;

use ::std::env;
use ::std::path::{
    Path,
    PathBuf
};

use ::io;
use ::misc::ResultExt;

pub const NAME: &'static str = "fie.toml";

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Platforms {
    #[serde(default)]
    pub gab: bool,
    #[serde(default)]
    pub twitter: bool,
    #[serde(default)]
    pub minds: bool
}

//If the whole section on Platforms is missing then we assume
//that all platforms are used.
//If section is present though, missing field means that user doesn't want platform.
impl Default for Platforms {
    fn default() -> Self {
        Platforms {
            gab: true,
            twitter: true,
            minds: true
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Token {
    pub key: String,
    pub secret: String
}
///Twitter configuration
#[derive(Deserialize, Debug)]
pub struct Twitter {
    pub consumer: Token,
    pub access: Token
}

///Gab configuration.
#[derive(Deserialize, Debug)]
pub struct Gab {
    ///Token to use for authorization.
    ///
    ///You can get it after logging into gab.io and examining your HTTP requests.
    pub token: String
}
#[derive(Deserialize, Debug)]
pub struct Minds {
    pub username: String,
    pub password: String
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub platforms: Platforms,
    pub gab: Gab,
    pub twitter: Twitter,
    pub minds: Minds
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        toml::from_str(io::read_file_to_string(path)?.as_str()).format_err("Invalid config file!")
    }

    pub fn default_config_path() -> PathBuf {
        let mut result = env::current_exe().unwrap();
        result.set_file_name(NAME);

        result
    }

    pub fn from_default_config() -> Result<Self, String> {
        let path = Self::default_config_path();

        Self::from_file(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::std::fs::File;
    use ::std::io::{
        BufReader,
        Read
    };

    #[test]
    fn deserialize() {
        let file = File::open("fie.toml").unwrap();
        let mut file = BufReader::new(file);

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();

        let result: Config = toml::from_str(&buffer).unwrap();
        println!("{:?}", result);
    }
}
