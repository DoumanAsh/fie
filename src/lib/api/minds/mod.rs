//!Gab API

const OAUTH2_URL: &'static str = "https://www.minds.com/api/v2/oauth/token";
const IMAGES_URL: &'static str = "https://www.minds.com/api/v1/media";
const POST_URL: &'static str = "https://www.minds.com/api/v1/newsfeed";

use crate::data::PostFlags;
use super::http::{multipart, GlobalRequest, Mime, Request, matsu};

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
    pub async fn new(config: crate::config::Minds) -> Result<Self, MindsError> {
        let req = Request::post(OAUTH2_URL).expect("To create request")
                                           .json(&Auth::new(&config.username, &config.password))
                                           .expect("To serialize json")
                                           .global()
                                           .send();

        let mut resp = match matsu!(req) {
            Ok(resp) => match resp {
                Ok(resp) => resp,
                Err(_) => return Err(MindsError::LoginFailed),
            },
            Err(_) => return Err(MindsError::LoginFailed),
        };

        if !resp.is_success() {
            return Err(MindsError::LoginFailed);
        }

        let oauth2 = match matsu!(resp.json::<Oauth2>()) {
            Ok(oauth2) => oauth2,
            Err(_) => return Err(MindsError::LoginFailed),
        };

        Ok(Self { token: oauth2.access_token })
    }

    ///Prepares image upload request.
    ///
    ///Future result contains `id` from `UploadResponse`
    pub async fn upload_image(&self, name: &str, mime: &Mime, data: &[u8]) -> Result<String, MindsError> {
        let mut form = multipart::Form::new();
        form.add_file_field("file".to_string(), name.to_string(), mime, data);

        let req = Request::post(IMAGES_URL).expect("To create request").bearer_auth(&self.token).multipart(form).global().send();

        // For image we wait twice of time
        // just to be sure
        let mut resp = match matsu!(req) {
            Ok(resp) => resp,
            Err(err) => match matsu!(matsu!(err)) {
                Ok(resp) => resp,
                Err(_) => return Err(MindsError::ImageUploadSendError)
            }
        }.map_err(|_| MindsError::ImageUploadSendError)?;


        if !resp.is_success() {
            return Err(MindsError::ImageUploadServerReject)
        }

        match matsu!(resp.json::<UploadResponse>()) {
            Ok(data) => Ok(data.guid),
            Err(_) => Err(MindsError::PostUploadInvalidResponse),
        }
    }

    ///Prepares post upload request.
    pub async fn post(&self, message: &str, media_attachments: Option<String>, flags: &PostFlags) -> Result<crate::data::PostId, MindsError> {
        let req = Request::post(POST_URL).expect("To create request")
                                         .bearer_auth(&self.token)
                                         .json(&Post::new(&message, &media_attachments, &flags))
                                         .expect("To serialzie post data")
                                         .global()
                                         .send();

        let mut resp = match matsu!(req) {
            Ok(resp) => resp,
            Err(err) => match matsu!(matsu!(err)) {
                Ok(resp) => resp,
                Err(_) => return Err(MindsError::PostUploadSendError)
            }
        }.map_err(|_| MindsError::PostUploadSendError)?;


        if !resp.is_success() {
            return Err(MindsError::PostUploadServerReject)
        }

        match matsu!(resp.json::<UploadResponse>()) {
            Ok(data) => Ok(data.guid.into()),
            Err(_) => Err(MindsError::PostUploadInvalidResponse),
        }
    }
}
