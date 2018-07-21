//! Describes common messages
extern crate actix;
extern crate mime_guess;

use io::Image;
use std::fmt;
use std::sync::Arc;

use self::actix::prelude::*;

/// Performs upload of image
pub struct UploadImage(pub Arc<Image>);

impl Message for UploadImage {
    type Result = Result<ResultImage, String>;
}

#[derive(Clone)]
/// Result of upload.
pub enum ResultImage {
    Id(u64),
    Guid(String),
}

impl ResultImage {
    pub fn id(self) -> u64 {
        match self {
            ResultImage::Id(result) => result,
            _ => panic!("Not an ID's type"),
        }
    }

    pub fn guid(self) -> String {
        match self {
            ResultImage::Guid(result) => result,
            _ => panic!("Not an Guid's type"),
        }
    }
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct PostFlags {
    /// Whether it is safe for work or not.
    #[serde(default)]
    pub nsfw: bool,
}

#[derive(Clone)]
/// Posts message on social media
pub struct PostMessage {
    /// Post's flags.
    pub flags: PostFlags,
    /// Message's text
    pub content: String,
    /// List of images to attach
    pub images: Option<Vec<ResultImage>>,
}

impl Message for PostMessage {
    type Result = Result<ResultMessage, String>;
}

/// Result's of message posting
pub enum ResultMessage {
    Id(u64),
    Guid(String),
}

impl fmt::Display for ResultMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ResultMessage::Id(id) => write!(f, "{}", id),
            &ResultMessage::Guid(ref id) => write!(f, "{}", id),
        }
    }
}
