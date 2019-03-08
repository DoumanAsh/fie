//!Minds data

use serde_derive::{Serialize, Deserialize};

use crate::data::PostFlags;

///Auth payload
#[derive(Serialize, Debug)]
pub struct Auth<'a> {
    grant_type: &'static str,
    client_id: &'static str,
    username: &'a str,
    password: &'a str,
}

impl<'a> Auth<'a> {
    ///Creates new payload
    pub fn new(username: &'a str, password: &'a str) -> Self {
        Auth {
            grant_type: "password",
            client_id: "mobile",
            username,
            password,
        }
    }
}

///Payload for successful authorization
#[derive(Deserialize, Debug)]
pub struct Oauth2 {
    ///Access token
    pub access_token: String,
    ///Expiration time(units?)
    pub expires_in: u64,
    ///Request's textual status
    pub status: String,
}

///Payload for post
#[derive(Serialize, Debug)]
pub struct Post<'a> {
    wire_threshold: Option<String>,
    message: &'a str,
    is_rich: u8,
    title: Option<String>,
    description: Option<String>,
    thumbnail: Option<String>,
    url: Option<String>,
    attachment_guid: &'a Option<String>,
    ///Whether content is safe for work or not
    pub mature: u8,
    access_id: u8,
}

impl<'a> Post<'a> {
    ///Creates new post
    pub fn new(message: &'a str, attachment_guid: &'a Option<String>, flags: &PostFlags) -> Self {
        Post {
            wire_threshold: None,
            message,
            is_rich: 0,
            title: None,
            description: None,
            thumbnail: None,
            url: None,
            attachment_guid,
            mature: flags.nsfw as u8,
            access_id: 2,
        }
    }
}

///Response to successful upload/post
#[derive(Deserialize, Debug)]
pub struct UploadResponse {
    ///Newly created entity ID
    pub guid: String,
}
