//! Actors to access twitter API
extern crate serde_urlencoded;
extern crate actix;
extern crate actix_web;
extern crate futures;

use self::actix_web::client::ClientRequest;
use self::actix_web::{http, HttpMessage};
use self::actix::prelude::*;
use self::futures::{future, Future};

use super::messages::{PostMessage, ResultImage, ResultMessage, UploadImage};
use config;
use misc::{ClientRequestBuilderExt, ClientRequestExt};

mod data;

/// Twitter actor
pub struct Twitter {
    oauth: data::Oauth,
    settings: config::Settings
}

impl Actor for Twitter {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {}
}

impl Twitter {
    pub fn new(config: config::Twitter, settings: config::Settings) -> Self {
        let oauth = data::Oauth::new(config);

        Self {oauth, settings}
    }
}

impl Handler<UploadImage> for Twitter {
    type Result = ResponseFuture<ResultImage, String>;

    fn handle(&mut self, msg: UploadImage, _: &mut Self::Context) -> Self::Result {
        use std::collections::HashMap;

        const IMAGES_URL: &'static str = "https://upload.twitter.com/1.1/media/upload.json";
        let mut req = ClientRequest::post(IMAGES_URL);
        let media = data::Media::from_bytes(&msg.0.mmap[..]);

        let auth_header = {
            let mut auth_params = HashMap::new();
            auth_params.insert("media_data", media.media_data.as_str());
            self.oauth.gen_auth(&http::Method::POST, IMAGES_URL, auth_params)
        };

        let media = match serde_urlencoded::to_string(media) {
            Ok(media) => media,
            Err(error) => return Box::new(future::err(format!("Twitter encoding error: {}", error))),
        };

        let req = req.set_default_headers()
                     .header(http::header::AUTHORIZATION, auth_header)
                     .header(http::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                     .content_length(media.as_bytes().len() as u64)
                     .body(media)
                     .map_err(|error| format!("Twitter upload creation error: {}", error));
        let req = match req {
            Ok(req) => req,
            Err(error) => return Box::new(future::err(format!("Twitter upload creation error: {}", error))),
        };

        let req = req.send_with_timeout(self.settings.timeout)
                     .map_err(|error| format!("Twitter upload error: {}", error))
                     .and_then(|response| match response.status().is_success() {
                         true => Ok(response),
                         false => Err(format!("Twitter server returned error code {}", response.status())),
                     })
                     .and_then(|response| response.json().map_err(|error| format!("Twitter upload reading error: {}", error)))
                     .map(|response: data::MediaResponse| ResultImage::Id(response.media_id));

        Box::new(req)
    }
}

impl Handler<PostMessage> for Twitter {
    type Result = ResponseFuture<ResultMessage, String>;

    fn handle(&mut self, msg: PostMessage, _: &mut Self::Context) -> Self::Result {
        use std::collections::HashMap;
        const POST_URL: &'static str = "https://api.twitter.com/1.1/statuses/update.json";
        let PostMessage { flags, content, images } = msg;

        let images = match images {
            Some(mut images) => Some(images.drain(..).map(|image| image.id()).collect::<Vec<_>>()),
            None => None,
        };
        let mut req = ClientRequest::post(POST_URL);
        let tweet = data::Tweet::new(content).nsfw(flags.nsfw);
        let tweet = match images.as_ref() {
            Some(images) => tweet.media_ids(&images),
            None => tweet
        };

        let auth_header = {
            if let Some(media_ids) = tweet.media_ids.as_ref() {
                let mut media_param = media_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
                let mut auth_params = HashMap::new();
                auth_params.insert("status", tweet.status.as_str());
                match tweet.possibly_sensitive {
                    true => auth_params.insert("possibly_sensitive", "true"),
                    false => auth_params.insert("possibly_sensitive", "false"),
                };
                auth_params.insert("media_ids", media_param.as_str());
                self.oauth.gen_auth(&http::Method::POST, POST_URL, auth_params)
            }
            else {
                let mut auth_params = HashMap::new();
                auth_params.insert("status", tweet.status.as_str());
                match tweet.possibly_sensitive {
                    true => auth_params.insert("possibly_sensitive", "true"),
                    false => auth_params.insert("possibly_sensitive", "false"),
                };
                self.oauth.gen_auth(&http::Method::POST, POST_URL, auth_params)
            }
        };

        println!("auth_header={}", &auth_header);

        let tweet = match serde_urlencoded::to_string(tweet) {
            Ok(tweet) => tweet,
            Err(error) => return Box::new(future::err(format!("Twitter encoding error: {}", error))),
        };

        println!("tweet payload={}", &tweet);

        let req = req.set_default_headers()
                     .header(http::header::AUTHORIZATION, auth_header)
                     .header(http::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                     .content_length(tweet.as_bytes().len() as u64)
                     .body(tweet)
                     .map_err(|error| format!("Twitter post creation error: {}", error));
        let req = match req {
            Ok(req) => req,
            Err(error) => return Box::new(future::err(format!("Twitter post creation error: {}", error))),
        };

        //TODO: Check why fails
        let req = req.send_with_timeout(self.settings.timeout)
                     .map_err(|error| format!("Twitter post error: {}", error))
                     .and_then(|response| match response.status().is_success() {
                         true => Ok(response),
                         false => Err(format!("Twitter post server returned error code {}", response.status())),
                     })
                     .and_then(|response| response.json().map_err(|error| format!("Twitter post reading error: {}", error)))
                     .map(|response: data::TweetResponse| ResultMessage::Id(response.id));

        Box::new(req)
    }
}
