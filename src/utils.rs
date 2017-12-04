use ::std::env;
use ::std::fs::{
    File
};
use std::path::{
    Path,
    PathBuf
};
use ::std::io;
use self::io::{
    BufReader,
    Read
};
use ::futures::future;
use ::mime_guess::{
    Mime,
    guess_mime_type
};

#[macro_export]
macro_rules! error_formatter {
    ($prefix:expr) => { |error| format!("{} Error: {}", $prefix, error) }
}

#[macro_export]
macro_rules! handle_bad_hyper_response {
    ($prefix:expr) => { |response| match response.status() {
        hyper::StatusCode::Ok => Ok(response),
        _ => Err(format!("{} Bad response. Status: {}", $prefix, response.status()))
    }}
}

use ::config;

pub struct Image {
    pub name: String,
    pub mime: Mime,
    pub content: Vec<u8>
}

///Opens image file and returns its content.
pub fn open_image<P: AsRef<Path>>(path: P) -> io::Result<Image> {
    let file = File::open(&path)?;
    let file_len = file.metadata()?.len();
    let mut file = BufReader::new(file);

    let name = path.as_ref().file_name().unwrap().to_string_lossy().to_string();
    let mime = guess_mime_type(path);
    let mut content = Vec::with_capacity(file_len as usize);
    file.read_to_end(&mut content)?;

    Ok(Image {
        name,
        mime,
        content
    })
}

///Retrieves configuration of Fie.
pub fn get_config() -> PathBuf {
    let mut result = env::current_exe().unwrap();

    result.set_file_name(config::NAME);

    result
}

pub fn empty_future_job() -> future::FutureResult<(), ()> {
    future::ok(())
}

///Reads content of file to string.
pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let file = File::open(path.as_ref()).map_err(error_formatter!("Cannot open config file."))?;
    let mut file = BufReader::new(file);

    let mut buffer = String::new();
    file.read_to_string(&mut buffer).map_err(error_formatter!("Unable to read config file."))?;

    Ok(buffer)
}
