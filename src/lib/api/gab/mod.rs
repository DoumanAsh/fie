//!Gab API

pub mod data;
mod error;

const LOGIN_URL: &'static str = "https://gab.com/auth/login";
const IMAGES_URL: &'static str = "https://gab.com/api/media-attachments/images";
const POST_URL: &'static str = "https://gab.com/posts";

use crate::data::PostFlags;
use data::*;
pub use error::GabError;

use super::http::{self, header, multipart, Future, IntoFuture, Mime, Request, AutoRuntime, AutoClient};

///Gab API
pub struct Gab {
    token: String,
}

impl Gab {
    fn extract_token(text: &str) -> Option<String> {
        // Look for _token
        let token_start = text.find("_token")?;
        let text = &text[token_start + 7..];
        // Look for value="{}"
        let value_start = text.find('"')? + 1;
        let text = &text[value_start..];
        let value_end = text.find('"')?;
        Some(text[..value_end].to_string())
    }

    ///Creates new instance by using password/login to authorize with site.
    pub fn new(config: crate::config::Gab) -> Result<Self, GabError> {
        let login = Request::get(LOGIN_URL).expect("To create login request").empty().send().finish();

        let login = match login {
            Ok(login) => login,
            Err(_) => {
                return Err(GabError::LoginFailed);
            },
        };

        if !login.is_success() {
            return Err(GabError::LoginFailed);
        }

        let cookies = login.cookies_jar().expect("To get cookie jar");
        let login = login.text().finish().expect("To get content of response");

        let token = match Self::extract_token(&login) {
            Some(token) => token,
            None => return Err(GabError::LoginFailed),
        };

        let token = {
            let auth = Auth::new(&config, &token);
            let post = Request::post(LOGIN_URL)
                .expect("To create login request")
                .set_cookie_jar(cookies)
                .json(&auth)
                .expect("To create json payload")
                .send()
                .finish();

            let post = match post {
                Ok(post) => post,
                Err(_) => {
                    return Err(GabError::LoginFailed);
                },
            };

            if !post.is_redirect() {
                return Err(GabError::LoginFailed);
            }

            let cookies = post.cookies_jar().expect("Get cookies");
            let redirect = match post.headers().get(header::LOCATION) {
                Some(redirect) => redirect,
                None => return Err(GabError::LoginFailed),
            };
            let redirect = redirect.to_str().expect("Convert location to string");
            let redirect = Request::get(redirect).expect("To create redirect request").set_cookie_jar(cookies).empty().send().finish();

            let redirect = match redirect {
                Ok(redirect) => redirect,
                Err(_) => {
                    return Err(GabError::LoginFailed);
                },
            };

            if !redirect.is_success() {
                return Err(GabError::LoginFailed);
            }

            let redirect = redirect.text().finish().expect("To get content of auth redirect");
            match Self::extract_token(&redirect) {
                Some(jwt_token) => match jwt_token == token {
                    false => jwt_token,
                    true => {
                        return Err(GabError::LoginFailed);
                    }
                },
                None => {
                    return Err(GabError::LoginFailed);
                },
            }
        };

        Ok(Self { token })
    }

    ///Prepares image upload request.
    ///
    ///Future result contains `id` from `UploadResponse`
    pub fn upload_image(&self, name: &str, mime: &Mime, data: &[u8]) -> impl Future<Item = String, Error = GabError> {
        let url = format!("{}?token={}", IMAGES_URL, &self.token);
        let mut form = multipart::Form::new();

        form.add_file_field("file".to_string(), name.to_string(), mime, data);

        let req = Request::post(url).expect("To create request").multipart(form).send();

        // For image we wait twice of time
        // just to be sure
        req.or_else(|resp| resp.retry(http::get_timeout()).into_future().flatten())
            .map_err(|_| GabError::ImageUploadSendError)
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => Err(GabError::ImageUploadServerReject),
            }).and_then(|response| response.json::<UploadResponse>().map_err(|_| GabError::PostUploadInvalidResponse))
            .map(|response| response.id)
    }

    ///Prepares post upload request.
    pub fn post(&self, message: &str, media_attachments: &[String], flags: &PostFlags) -> impl Future<Item = crate::data::PostId, Error = GabError> {
        let req = Request::post(POST_URL)
            .expect("To create request")
            .bearer_auth(self.token.as_str())
            .json(&PostData::new(&message, &media_attachments, &flags))
            .expect("To serialzie post data")
            .send();

        req.map_err(|_| GabError::PostUploadSendError)
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => Err(GabError::PostUploadServerReject),
            }).and_then(|resp| resp.json::<PostResponse>().map_err(|_| GabError::PostUploadInvalidResponse))
            .map(|resp| resp.post.id.into())
    }
}
