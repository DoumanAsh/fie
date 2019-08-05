//!Image utilities

use memmap::{Mmap, MmapOptions};
use mime_guess::{Mime};

use std::fs::File;
use std::io;
use std::path::Path;

///Loaded image.
///
///Internally it uses `memmap`
pub struct Image {
    ///Image's file name
    pub name: String,
    ///Mime of Image
    pub mime: Mime,
    _file: File,
    pub(crate) mmap: Mmap,
}

impl Image {
    ///Opens image in specified file
    ///
    ///Doesn't verify whether it is actually image or not.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let _file = File::open(&path)?;
        let mmap = unsafe { MmapOptions::new().map(&_file)? };

        let name = path.as_ref().file_name().unwrap().to_string_lossy().to_string();
        let mime = mime_guess::from_path(path).first_or_octet_stream();

        Ok(Image { name, mime, _file, mmap })
    }
}
