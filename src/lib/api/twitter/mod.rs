//!Twitter API

pub mod data;
mod error;

use std::collections::HashMap;

use crate::config;
use super::http::{self, GlobalRequest, Mime, Request, matsu};

use crate::data::PostFlags;
pub use error::TwitterError;

const IMAGES_URL: &'static str = "https://upload.twitter.com/1.1/media/upload.json";
const POST_URL: &'static str = "https://api.twitter.com/1.1/statuses/update.json";

///Twitter API
pub struct Twitter {
    oauth: data::Oauth,
}

impl Twitter {
    ///Verifies and creates twitter API instance
    pub fn new(config: config::Twitter) -> Result<Self, TwitterError> {
        if config.consumer.key.len() == 0 || config.consumer.secret.len() == 0 || config.access.key.len() == 0 || config.access.secret.len() == 0 {
            Err(TwitterError::InvalidAuthData)
        } else {
            let oauth = data::Oauth::new(config);
            Ok(Self { oauth })
        }
    }

    ///Prepares image upload request.
    ///
    ///Result contains `id` from `MediaResponse`
    pub async fn upload_image(&self, _name: &str, _mime: &Mime, data: &[u8]) -> Result<u64, TwitterError> {
        let media = data::Media::from_bytes(data);

        let auth_header = {
            let mut auth_params = HashMap::new();
            auth_params.insert("media_data", media.media_data.as_str());
            self.oauth.gen_auth(&http::Method::POST, IMAGES_URL, auth_params)
        };

        let req = Request::post(IMAGES_URL).expect("To create request")
                                           .set_header(http::header::AUTHORIZATION, auth_header)
                                           .form(&media)
                                           .expect("To finalize request")
                                           .global()
                                           .send();

        let mut resp = match matsu!(req) {
            Ok(resp) => resp,
            Err(err) => match matsu!(matsu!(err)) {
                Ok(resp) => resp,
                Err(_) => return Err(TwitterError::ImageUploadSendError)
            }
        }.map_err(|_| TwitterError::ImageUploadSendError)?;


        if !resp.is_success() {
            return Err(TwitterError::ImageUploadServerReject)
        }

        match matsu!(resp.json::<data::MediaResponse>()) {
            Ok(data) => Ok(data.media_id),
            Err(_) => Err(TwitterError::PostUploadInvalidResponse),
        }
    }

    ///Prepares post upload request.
    pub async fn post(&self, message: &str, media_attachments: &[u64], flags: &PostFlags) -> Result<crate::data::PostId, TwitterError> {
        let tweet = data::Tweet::new(message).nsfw(flags.nsfw).media_ids(media_attachments);

        let auth_header = {
            let mut auth_params = HashMap::new();
            auth_params.insert("status", tweet.status);
            match tweet.possibly_sensitive {
                true => auth_params.insert("possibly_sensitive", "true"),
                false => auth_params.insert("possibly_sensitive", "false"),
            };
            if let Some(ids) = tweet.media_ids.as_ref() {
                auth_params.insert("media_ids", ids);
            }
            self.oauth.gen_auth(&http::Method::POST, POST_URL, auth_params)
        };

        let req = Request::post(POST_URL).expect("To create request")
                                         .set_header(http::header::AUTHORIZATION, auth_header)
                                         .form(&tweet)
                                         .expect("To create tweet data")
                                         .global()
                                         .send();

        let mut resp = match matsu!(req) {
            Ok(resp) => resp,
            Err(err) => match matsu!(matsu!(err)) {
                Ok(resp) => resp,
                Err(_) => return Err(TwitterError::PostUploadSendError)
            }
        }.map_err(|_| TwitterError::PostUploadSendError)?;


        if !resp.is_success() {
            return Err(TwitterError::PostUploadServerReject)
        }

        match matsu!(resp.json::<data::TweetResponse>()) {
            Ok(data) => Ok(data.id.into()),
            Err(_) => Err(TwitterError::PostUploadInvalidResponse),
        }
    }
}
