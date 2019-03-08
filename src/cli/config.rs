use std::path::{Path, PathBuf};
use std::io::{self, Read};
use std::fs;
use std::env;

use serde::de::{DeserializeOwned};

pub const NAME: &str = "fie.toml";

pub fn load_from_file<T: DeserializeOwned>(path: &Path) -> io::Result<T> {
    let mut file = fs::File::open(&path).map_err(|error| io::Error::new(io::ErrorKind::Other, format!("{}: {}", path.display(), error)))?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).map_err(|error| io::Error::new(io::ErrorKind::Other, format!("{}: {}", path.display(), error)))?;
    toml::from_str(&buffer).map_err(|error| io::Error::new(io::ErrorKind::Other, format!("Invalid config: {}", error)))
}

pub trait FileSystemLoad: DeserializeOwned {
    fn path() -> io::Result<PathBuf> {
        match env::current_exe() {
            Ok(mut result) => {
                result.set_file_name(NAME);
                if result.exists() {
                    return Ok(result);
                }
            },
            Err(_) => (),
        }

        match dirs::home_dir() {
            Some(mut result) => {
                result.push(".fie");
                result.push(NAME);
                println!("Look: {}", result.display());
                if result.exists() {
                    return Ok(result)
                }
            }
            None => (),
        }

        Err(io::Error::new(io::ErrorKind::Other, "Unable to find configuration file"))
    }

    fn load() -> io::Result<Self> {
        let path = Self::path()?;
        load_from_file(&path)
    }
}

impl FileSystemLoad for fie::config::Config {}
