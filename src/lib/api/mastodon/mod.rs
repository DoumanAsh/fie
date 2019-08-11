//!Mastodon API

use super::http::{multipart, Uri, GlobalRequest, Mime, Request, matsu};
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
    ///Result contains `id` from `EntityId`
    pub async fn upload_image(&self, name: &str, mime: &Mime, data: &[u8]) -> Result<String, MastodonError> {
        let url = format!("https://{}/api/v1/media", &self.config.host);
        let mut form = multipart::Form::new();

        form.add_file_field("file".to_string(), name.to_string(), mime, data);

        let req = Request::post(url).expect("To create request").bearer_auth(self.config.access_token.as_str()).multipart(form).global().send();

        // For image we wait twice of time
        // just to be sure
        let mut resp = match matsu!(req) {
            Ok(resp) => resp,
            Err(err) => match matsu!(matsu!(err)) {
                Ok(resp) => resp,
                Err(_) => return Err(MastodonError::ImageUploadSendError)
            }
        }.map_err(|_| MastodonError::ImageUploadSendError)?;

        if !resp.is_success() {
            return Err(MastodonError::ImageUploadServerReject)
        }

        match matsu!(resp.json::<data::EntityId>()) {
            Ok(data) => Ok(data.id),
            Err(_) => Err(MastodonError::PostUploadInvalidResponse),
        }

    }

    ///Prepares post upload request.
    pub async fn post(&self, message: &str, media_attachments: &[String], flags: &PostFlags) -> Result<crate::data::PostId, MastodonError> {
        let url = format!("https://{}/api/v1/statuses", &self.config.host);
        let req = Request::post(url).expect("To create request")
                                    .bearer_auth(self.config.access_token.as_str())
                                    .json(&data::NewStatus::new(&message, &media_attachments, &flags))
                                    .expect("To serialzie post data")
                                    .global()
                                    .send();

        let mut resp = match matsu!(req) {
            Ok(resp) => resp,
            Err(err) => match matsu!(matsu!(err)) {
                Ok(resp) => resp,
                Err(_) => return Err(MastodonError::PostUploadSendError)
            }
        }.map_err(|_| MastodonError::PostUploadSendError)?;


        if !resp.is_success() {
            return Err(MastodonError::PostUploadServerReject)
        }

        match matsu!(resp.json::<data::EntityId>()) {
            Ok(data) => Ok(data.id.into()),
            Err(_) => Err(MastodonError::PostUploadInvalidResponse),
        }
    }
}
