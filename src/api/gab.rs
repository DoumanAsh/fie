use crate::config;
use crate::data::PostFlags;
use crate::http::{self, header, multipart, AutoClient, AutoRuntime, Future, IntoFuture, Mime, Request};

const LOGIN_URL: &'static str = "https://gab.com/auth/login";
const IMAGES_URL: &'static str = "https://gab.com/api/media-attachments/images";
const POST_URL: &'static str = "https://gab.com/posts";

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

    pub fn new(config: config::Gab) -> Option<Self> {
        let login = Request::get(LOGIN_URL).expect("To create login request").empty().send().finish();

        let login = match login {
            Ok(login) => login,
            Err(error) => {
                eprintln!("Gab: Unable to get login. Error: {}", error);
                return None;
            },
        };

        if !login.is_success() {
            eprintln!("Gab: Failed to get login page. Status code={}", login.status());
            return None;
        }

        let cookies = login.cookies_jar().expect("To get cookie jar");
        let login = login.text().finish().expect("To get content of response");

        let token = Self::extract_token(&login)?;

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
                Err(error) => {
                    eprintln!("Gab: Failed to authorize. Error={}", error);
                    return None;
                },
            };

            if !post.is_redirect() {
                eprintln!("Gab: Failed to authorize. Status code={}", post.status());
                return None;
            }

            let cookies = post.cookies_jar().expect("Get cookies");
            let redirect = post.headers().get(header::LOCATION)?;
            let redirect = redirect.to_str().expect("Convert location to string");
            let redirect = Request::get(redirect).expect("To create redirect request").set_cookie_jar(cookies).empty().send().finish();

            let redirect = match redirect {
                Ok(redirect) => redirect,
                Err(error) => {
                    eprintln!("Gab: Failed to redirect on auth success. Error={}", error);
                    return None;
                },
            };

            if !redirect.is_success() {
                eprintln!("Gab: Failed to follow auth redirect. Status code={}", redirect.status());
                return None;
            }

            let redirect = redirect.text().finish().expect("To get content of auth redirect");
            match Self::extract_token(&redirect) {
                Some(jwt_token) => match jwt_token == token {
                    false => jwt_token,
                    true => {
                        eprintln!("Gab: Failed to authorize");
                        return None;
                    }
                },
                None => {
                    eprintln!("Gab: Failed to authorize");
                    return None;
                },
            }
        };

        Some(Self { token })
    }

    pub fn upload_image(&self, name: &str, mime: &Mime, data: &[u8]) -> impl Future<Item = String, Error = ()> {
        let url = format!("{}?token={}", IMAGES_URL, &self.token);
        let mut form = multipart::Form::new();

        form.add_file_field("file".to_string(), name.to_string(), mime, data);

        let req = Request::post(url).expect("To create request").multipart(form).send();

        // For image we wait twice of time
        // just to be sure
        req.or_else(|resp| resp.retry(http::get_timeout()).into_future().flatten())
            .map_err(|error| eprintln!("Gab: uploading image Error={}", error))
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => {
                    eprintln!("Gab: failed to upload image. Status code={}", resp.status());
                    Err(())
                },
            }).and_then(|response| response.json::<UploadResponse>().map_err(|error| eprintln!("Gab upload reading error: {}", error)))
            .map(|response| response.id)
    }

    pub fn post(&self, message: &str, media_attachments: &[String], flags: &PostFlags) -> impl Future<Item = (), Error = ()> {
        let req = Request::post(POST_URL)
            .expect("To create request")
            .bearer_auth(self.token.as_str())
            .json(&PostData::new(&message, &media_attachments, &flags))
            .expect("To serialzie post data")
            .send();

        req.map_err(|error| eprintln!("Gab: post error. Error={}", error))
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => {
                    eprintln!("Gab: failed to post. Status code={}", resp.status());
                    Err(())
                },
            }).and_then(|resp| resp.json::<PostResponse>().map_err(|error| eprintln!("Gab: Invalid response. Error={}", error)))
            .map(|resp| println!("GAB(id={}) OK", resp.post.id))
    }
}

#[derive(Serialize, Debug)]
struct Auth<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub remember: &'static str,
    pub _token: &'a str,
}

impl<'a> Auth<'a> {
    fn new(config: &'a config::Gab, _token: &'a str) -> Self {
        Self {
            username: config.username.as_str(),
            password: config.password.as_str(),
            remember: "on",
            _token,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct UploadResponse {
    pub id: String,
}

#[derive(Serialize, Debug)]
pub struct PostData<'a> {
    body: &'a str,
    pub reply_to: &'a str,
    pub is_quote: u8,
    pub is_html: u8,
    pub nsfw: u8,
    pub is_premium: u8,
    pub _method: &'a str,
    pub gif: &'a str,
    pub category: Option<&'a str>,
    pub topic: Option<&'a str>,
    pub share_twitter: bool,
    pub share_facebook: bool,
    pub media_attachments: &'a [String],
}

impl<'a> PostData<'a> {
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
pub struct Post {
    pub id: u64,
}
#[derive(Deserialize, Debug)]
pub struct PostResponse {
    pub post: Post,
}
