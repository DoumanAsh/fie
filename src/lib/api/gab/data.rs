//!Gab data

use serde_derive::{Serialize, Deserialize};

use crate::data::PostFlags;

#[derive(Serialize, Debug)]
///Authorization payload
pub struct Auth<'a> {
    ///Username
    pub username: &'a str,
    ///Password
    pub password: &'a str,
    ///Remember, whatever it is, just set to "on"
    pub remember: &'static str,
    ///Token taken from login page
    pub _token: &'a str,
}

impl<'a> Auth<'a> {
    ///Creates new instance
    pub fn new(config: &'a crate::config::Gab, _token: &'a str) -> Self {
        Self {
            username: config.username.as_str(),
            password: config.password.as_str(),
            remember: "on",
            _token,
        }
    }
}

#[derive(Deserialize, Debug)]
///Image upload response
pub struct UploadResponse {
    ///Id
    pub id: String,
}

#[derive(Serialize, Debug)]
///Post payload
pub struct PostData<'a> {
    body: &'a str,
    pub(crate) reply_to: &'a str,
    pub(crate) is_quote: u8,
    pub(crate) is_html: u8,
    pub(crate) nsfw: u8,
    pub(crate) is_premium: u8,
    pub(crate) _method: &'a str,
    pub(crate) gif: &'a str,
    pub(crate) category: Option<&'a str>,
    pub(crate) topic: Option<&'a str>,
    pub(crate) share_twitter: bool,
    pub(crate) share_facebook: bool,
    pub(crate) media_attachments: &'a [String],
}

impl<'a> PostData<'a> {
    ///Creates new instance
    pub fn new(message: &'a str, media_attachments: &'a [String], flags: &PostFlags) -> Self {
        Self {
            body: message,
            reply_to: "",
            is_quote: 0,
            is_html: 0,
            nsfw: flags.nsfw as u8,
            is_premium: 0,
            _method: "post",
            gif: "",
            category: None,
            topic: None,
            share_twitter: false,
            share_facebook: false,
            media_attachments,
        }
    }
}

#[derive(Deserialize, Debug)]
///Post description
pub struct Post {
    ///ID
    pub id: String,
}

#[derive(Deserialize, Debug)]
///Response to post creation
pub struct PostResponse {
    ///Post description
    pub post: Post,
}
