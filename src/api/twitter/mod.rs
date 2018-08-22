use data::PostFlags;

mod data;

use std::collections::HashMap;

use config;
use http;
use http::{AutoClient, Future, IntoFuture, Mime, Request};

const IMAGES_URL: &'static str = "https://upload.twitter.com/1.1/media/upload.json";
const POST_URL: &'static str = "https://api.twitter.com/1.1/statuses/update.json";

pub struct Twitter {
    oauth: data::Oauth,
}

impl Twitter {
    pub fn new(config: config::Twitter) -> Option<Self> {
        let oauth = data::Oauth::new(config);

        Some(Self { oauth })
    }

    pub fn upload_image(&self, _name: &str, _mime: &Mime, data: &[u8]) -> impl Future<Item = u64, Error = ()> {
        let media = data::Media::from_bytes(data);

        let auth_header = {
            let mut auth_params = HashMap::new();
            auth_params.insert("media_data", media.media_data.as_str());
            self.oauth.gen_auth(&http::Method::POST, IMAGES_URL, auth_params)
        };

        let req = Request::post(IMAGES_URL)
            .expect("To create request")
            .set_header(http::header::AUTHORIZATION, auth_header)
            .form(&media)
            .expect("To finalize request")
            .send();

        // For image we wait twice of time
        // just to be sure
        req.or_else(|resp| resp.retry(http::get_timeout()).into_future().flatten())
            .map_err(|error| eprintln!("Twitter: Upload error: {:?}", error))
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => {
                    eprintln!("Twitter: failed to upload image. Status code={}", resp.status());
                    Err(())
                },
            }).and_then(|rsp| rsp.json().map_err(|error| eprintln!("Twitter: invalid response to upload. Error: {:?}", error)))
            .map(|response: data::MediaResponse| response.media_id)
    }

    pub fn post(&self, message: &str, media_attachments: &[u64], flags: &PostFlags) -> impl Future<Item = (), Error = ()> {
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

        let req = Request::post(POST_URL)
            .expect("To create request")
            .set_header(http::header::AUTHORIZATION, auth_header)
            .form(&tweet)
            .expect("To create tweet data")
            .send();

        req.map_err(|error| eprintln!("Twitter: Post error: {:?}", error))
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => {
                    eprintln!("Twitter: failed to post tweet. Status code={}", resp.status());
                    Err(())
                },
            }).and_then(|rsp| rsp.json().map_err(|error| eprintln!("Twitter: invalid response to post. Error: {:?}", error)))
            .map(|response: data::TweetResponse| println!("Tweeter(Id={}) OK", response.id))
    }
}
