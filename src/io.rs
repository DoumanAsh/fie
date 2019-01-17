use memmap::{Mmap, MmapOptions};
use mime_guess::{guess_mime_type, Mime};

use crate::misc::ResultExt;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let mut file = File::open(path.as_ref()).format_err("Cannot open config file.")?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).format_err("Unable to read config file.")?;

    Ok(buffer)
}

pub struct Image {
    pub name: String,
    pub mime: Mime,
    pub file: File,
    pub mmap: Mmap,
}

impl Image {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(&path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        let name = path.as_ref().file_name().unwrap().to_string_lossy().to_string();
        let mime = guess_mime_type(path);

        Ok(Image { name, mime, file, mmap })
    }
}
