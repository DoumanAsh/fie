//!Gab API

const OAUTH2_URL: &'static str = "https://www.minds.com/api/v2/oauth/token";
const IMAGES_URL: &'static str = "https://www.minds.com/api/v1/media";
const POST_URL: &'static str = "https://www.minds.com/api/v1/newsfeed";

use crate::data::PostFlags;
use super::http::{self, multipart, Future, IntoFuture, Mime, Request, AutoRuntime, AutoClient};

pub mod data;
mod error;

use data::*;
pub use error::MindsError;

///Minds API
pub struct Minds {
    token: String,
}

impl Minds {
    ///Creates new instances by attempting to login and get access token.
    pub fn new(config: crate::config::Minds) -> Result<Self, MindsError> {
        let req = Request::post(OAUTH2_URL)
            .expect("To create request")
            .json(&Auth::new(&config.username, &config.password))
            .expect("To serialize json")
            .send()
            .finish();

        let req = match req {
            Ok(req) => req,
            Err(_) => return Err(MindsError::LoginFailed),
        };

        if !req.is_success() {
            return Err(MindsError::LoginFailed);
        }

        let oauth2 = match req.json::<Oauth2>().finish() {
            Ok(oauth2) => oauth2,
            Err(_) => return Err(MindsError::LoginFailed),
        };

        Ok(Self { token: oauth2.access_token })
    }

    ///Prepares image upload request.
    ///
    ///Future result contains `id` from `UploadResponse`
    pub fn upload_image(&self, name: &str, mime: &Mime, data: &[u8]) -> impl Future<Item = String, Error = MindsError> {
        let mut form = multipart::Form::new();
        form.add_file_field("file".to_string(), name.to_string(), mime, data);

        let req = Request::post(IMAGES_URL).expect("To create request").bearer_auth(&self.token).multipart(form).send();

        // For image we wait twice of time
        // just to be sure
        req.or_else(|resp| resp.retry(http::get_timeout()).into_future().flatten())
           .map_err(|_| MindsError::ImageUploadSendError)
           .and_then(|resp| match resp.is_success() {
               true => Ok(resp),
               false => Err(MindsError::ImageUploadServerReject),
           }).and_then(|response| response.json::<UploadResponse>().map_err(|_| MindsError::PostUploadInvalidResponse))
           .map(|response| response.guid)
    }

    ///Prepares post upload request.
    pub fn post(&self, message: &str, media_attachments: Option<String>, flags: &PostFlags) -> impl Future<Item = crate::data::PostId, Error = MindsError> {
        let req = Request::post(POST_URL)
            .expect("To create request")
            .bearer_auth(&self.token)
            .json(&Post::new(&message, &media_attachments, &flags))
            .expect("To serialzie post data")
            .send();

        req.map_err(|_| MindsError::PostUploadSendError)
           .and_then(|resp| match resp.is_success() {
               true => Ok(resp),
               false => Err(MindsError::PostUploadServerReject),
           }).and_then(|resp| resp.json::<UploadResponse>().map_err(|_| MindsError::PostUploadInvalidResponse))
           .map(|resp| resp.guid.into())
    }
}
