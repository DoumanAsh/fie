//!Mastodon API

use super::http::{self, Uri, multipart, Future, IntoFuture, Mime, Request, AutoClient};
use crate::data::PostFlags;

pub mod data;
mod error;

pub use error::MastodonError;

///Mastodon API
pub struct Mastodon {
    config: crate::config::Mastodon,
}

impl Mastodon {
    ///Verifies configuration and creates new instances.
    pub fn new(config: crate::config::Mastodon) -> Result<Self, MastodonError> {
        match config.host.parse::<Uri>() {
            Ok(_) => (),
            Err(_) => {
                return Err(MastodonError::InvalidHostUri);
            }
        }

        if config.access_token.len() == 0 {
            return Err(MastodonError::InvalidToken);
        }

        Ok(Self {
            config
        })
    }

    ///Prepares image upload request.
    ///
    ///Future result contains `id` from `EntityId`
    pub fn upload_image(&self, name: &str, mime: &Mime, data: &[u8]) -> impl Future<Item = String, Error = MastodonError> {
        let url = format!("https://{}/api/v1/media", &self.config.host);
        let mut form = multipart::Form::new();

        form.add_file_field("file".to_string(), name.to_string(), mime, data);

        let req = Request::post(url).expect("To create request").bearer_auth(self.config.access_token.as_str()).multipart(form).send();

        // For image we wait twice of time
        // just to be sure
        req.or_else(|resp| resp.retry(http::get_timeout()).into_future().flatten())
            .map_err(|_| MastodonError::ImageUploadSendError)
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => Err(MastodonError::ImageUploadServerReject),
            }).and_then(|response| response.json::<data::EntityId>().map_err(|_| MastodonError::ImageUploadInvalidResponse))
            .map(|response| response.id)
    }

    ///Prepares post upload request.
    pub fn post(&self, message: &str, media_attachments: &[String], flags: &PostFlags) -> impl Future<Item = crate::data::PostId, Error = MastodonError> {
        let url = format!("https://{}/api/v1/statuses", &self.config.host);
        let req = Request::post(url).expect("To create request")
                                    .bearer_auth(self.config.access_token.as_str())
                                    .json(&data::NewStatus::new(&message, &media_attachments, &flags))
                                    .expect("To serialzie post data")
                                    .send();

        req.map_err(|_| MastodonError::PostUploadSendError)
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => Err(MastodonError::PostUploadServerReject),
            }).and_then(|resp| resp.json::<data::EntityId>().map_err(|_| MastodonError::PostUploadInvalidResponse))
            .map(|resp| resp.id.into())
    }
}
