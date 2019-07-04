//!Gab API

pub mod data;
mod error;

const IMAGES_URL: &'static str = "https://gab.com/api/v1/media";
const POST_URL: &'static str = "https://gab.com/api/v1/statuses";

use crate::data::PostFlags;
use data::*;
pub use error::GabError;

use super::http::{self, multipart, Future, IntoFuture, Mime, Request, AutoClient};

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
    ///Future result contains `id` from `UploadResponse`
    pub fn upload_image(&self, name: &str, mime: &Mime, data: &[u8]) -> impl Future<Item = String, Error = GabError> {
        let mut form = multipart::Form::new();

        form.add_file_field("file".to_string(), name.to_string(), mime, data);

        let req = Request::post(IMAGES_URL).expect("To create request").bearer_auth(self.token.as_str()).multipart(form).send();

        // For image we wait twice of time
        // just to be sure
        req.or_else(|resp| resp.retry(http::get_timeout()).into_future().flatten())
           .map_err(|_| GabError::ImageUploadSendError)
           .and_then(|resp| match resp.is_success() {
               true => Ok(resp),
               false => Err(GabError::ImageUploadServerReject),
           }).and_then(|response| response.json::<EntityId>().map_err(|_| GabError::PostUploadInvalidResponse))
           .map(|response| response.id)
    }

    ///Prepares post upload request.
    pub fn post(&self, message: &str, media_attachments: &[String], flags: &PostFlags) -> impl Future<Item = crate::data::PostId, Error = GabError> {
        let req = Request::post(POST_URL).expect("To create request")
                                         .bearer_auth(self.token.as_str())
                                         .json(&NewStatus::new(&message, &media_attachments, &flags))
                                         .expect("To serialzie post data")
                                         .send();

        req.map_err(|_| GabError::PostUploadSendError)
           .and_then(|resp| match resp.is_success() {
               true => Ok(resp),
               false => Err(GabError::PostUploadServerReject),
           }).and_then(|resp| resp.json::<EntityId>().map_err(|_| GabError::PostUploadInvalidResponse))
           .map(|resp| resp.id.into())
    }
}
