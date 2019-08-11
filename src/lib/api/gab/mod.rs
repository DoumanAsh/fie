//!Gab API

pub mod data;
mod error;

const IMAGES_URL: &'static str = "https://gab.com/api/v1/media";
const POST_URL: &'static str = "https://gab.com/api/v1/statuses";

use crate::data::PostFlags;
use data::*;
pub use error::GabError;

use super::http::{multipart, GlobalRequest, Mime, Request, matsu};

///Gab API
pub struct Gab {
    token: String,
}

impl Gab {
    ///Creates new instance by using password/login to authorize with site.
    pub fn new(config: crate::config::Gab) -> Result<Self, GabError> {
        Ok(Self {
            token: config.access_token,
        })
    }

    ///Prepares image upload request.
    ///
    ///Result contains `id` from `EntityId`
    pub async fn upload_image(&self, name: &str, mime: &Mime, data: &[u8]) -> Result<String, GabError> {
        let mut form = multipart::Form::new();

        form.add_file_field("file".to_string(), name.to_string(), mime, data);

        let req = Request::post(IMAGES_URL).expect("To create request").bearer_auth(self.token.as_str()).multipart(form).global().send();

        // For image we wait twice of time
        // just to be sure
        let mut resp = match matsu!(req) {
            Ok(resp) => resp,
            Err(err) => match matsu!(matsu!(err)) {
                Ok(resp) => resp,
                Err(_) => return Err(GabError::ImageUploadSendError)
            }
        }.map_err(|_| GabError::ImageUploadSendError)?;

        if !resp.is_success() {
            return Err(GabError::ImageUploadServerReject)
        }

        match matsu!(resp.json::<EntityId>()) {
            Ok(data) => Ok(data.id),
            Err(_) => Err(GabError::PostUploadInvalidResponse),
        }
    }

    ///Prepares post upload request.
    pub async fn post(&self, message: &str, media_attachments: &[String], flags: &PostFlags) -> Result<crate::data::PostId, GabError> {
        let req = Request::post(POST_URL).expect("To create request")
                                         .bearer_auth(self.token.as_str())
                                         .json(&NewStatus::new(&message, &media_attachments, &flags))
                                         .expect("To serialzie post data")
                                         .global()
                                         .send();
        let mut resp = match matsu!(req) {
            Ok(resp) => resp,
            Err(err) => match matsu!(matsu!(err)) {
                Ok(resp) => resp,
                Err(_) => return Err(GabError::PostUploadSendError)
            }
        }.map_err(|_| GabError::PostUploadSendError)?;


        if !resp.is_success() {
            return Err(GabError::PostUploadServerReject)
        }

        match matsu!(resp.json::<EntityId>()) {
            Ok(data) => Ok(data.id.into()),
            Err(_) => Err(GabError::PostUploadInvalidResponse),
        }
    }
}
