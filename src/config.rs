//!Configuration module
use ::std::fs::{
    File
};
use ::std::path::{
    Path
};
use ::std::io::{
    BufReader,
    Read
};

use ::toml;

pub const NAME: &'static str = "fie.toml";

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
pub struct Config {
    pub gab: Gab,
    pub twitter: Twitter
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(error_formatter!("Cannot open config file."))?;
        let mut file = BufReader::new(file);

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).map_err(error_formatter!("Unable to read config file."))?;

        toml::from_str(&buffer).map_err(error_formatter!("Invalid config file!"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
