//!Data module

pub mod image;

pub use image::Image;

use serde_derive::{Deserialize};

use std::fmt;

///Flags for text posts
#[derive(Deserialize, Default, Debug, Clone)]
pub struct PostFlags {
    /// Whether post is safe for work or not.
    #[serde(default)]
    pub nsfw: bool,
}

///Describes text post
#[derive(Deserialize, Debug)]
pub struct Post {
    ///Post's content
    pub message: String,
    ///Hashtags to add
    pub tags: Vec<String>,
    ///Attachments
    pub images: Vec<String>,
    #[serde(default)]
    ///Flags
    pub flags: PostFlags,
}

///Generic Post ID.
///
///As different types are used by various social medias APIs
///this single type is supposed to hold all possible variants
#[derive(Clone, Debug)]
pub enum PostId {
    ///ID as integer
    Numeric(u64),
    ///ID as String
    String(String),
    ///ID as static str
    Str(&'static str)
}

impl fmt::Display for PostId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &PostId::Numeric(id) => write!(f, "{}", id),
            &PostId::String(ref id) => write!(f, "{}", id),
            &PostId::Str(id) => write!(f, "{}", id),
        }
    }
}

impl Into<PostId> for u64 {
    fn into(self) -> PostId {
        PostId::Numeric(self)
    }
}

impl Into<PostId> for String {
    fn into(self) -> PostId {
        PostId::String(self)
    }
}

impl Into<PostId> for &'static str {
    fn into(self) -> PostId {
        PostId::Str(self)
    }
}
///Creates string of multiple hashtags
pub fn join_hash_tags<'a, I: AsRef<str> + 'a, T: IntoIterator<Item = I>>(tags: T) -> String {
    let mut result = String::new();

    for tag in tags {
        result.push_str(&format!("#{} ", tag.as_ref()));
    }

    // remove last white space
    let _ = result.pop();

    result
}
