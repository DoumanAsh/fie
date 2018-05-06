//! Actors to access Gab API

extern crate futures;
extern crate actix;
extern crate actix_web;
extern crate serde_json;

use self::futures::{
    future,
    Future
};
use self::actix::prelude::*;
use self::actix_web::HttpMessage;
use self::actix_web::client::ClientRequest;

use ::config;
use ::misc::{
    ClientRequestBuilderExt,
    ClientRequestExt
};
use super::messages::{
    UploadImage,
    ResultImage,
    PostMessage,
    ResultMessage,
    PostFlags
};

///Gab actor
pub struct Gab {
    config: config::Gab
}

impl Actor for Gab {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
    }
}

impl Gab {
    pub fn new(config: config::Gab) -> Self {
        Self {
            config
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct UploadResponse {
    pub id: String
}

impl Handler<UploadImage> for Gab {
    type Result = ResponseFuture<ResultImage, String>;

    fn handle(&mut self, msg: UploadImage, _: &mut Self::Context) -> Self::Result {
        const IMAGES_URL: &'static str = "https://gab.ai/api/media-attachments/images";

        let name = &msg.0.name;
        let mime = &msg.0.mime;
        let data = &msg.0.mmap[..];

        let mut req = ClientRequest::post(format!("{}?token={}", IMAGES_URL, &self.config.token));

        let req = match req.set_default_headers().set_multipart_body(&name, &mime, &data) {
            Ok(req) => req,
            Err(error) => return Box::new(future::err(error))
        };

        let req = req.send_ext().map_err(|error| format!("Gab upload error: {}", error))
                     .and_then(|response| match response.status().is_success() {
                         true => Ok(response),
                         false => Err(format!("Gab server returned error code {}", response.status())),
                     })
                     .and_then(|response| response.json().map_err(|error| format!("Gab upload reading error: {}", error)))
                     .map(|response: UploadResponse| ResultImage::Guid(response.id));

        Box::new(req)
    }
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
    pub media_attachments: &'a [String]
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
            media_attachments
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Post {
    pub id: u64
}
#[derive(Deserialize, Debug)]
pub struct PostResponse {
    pub post: Post
}

impl Handler<PostMessage> for Gab {
    type Result = ResponseFuture<ResultMessage, String>;

    fn handle(&mut self, msg: PostMessage, _: &mut Self::Context) -> Self::Result {
        const POST_URL: &'static str = "https://gab.ai/posts";

        let PostMessage{flags, content, images} = msg;

        let mut req = ClientRequest::post(POST_URL);

        let images = match images {
            Some(mut images) => images.drain(..).map(|image| image.guid()).collect(),
            None => Vec::new()
        };

        let req = req.set_default_headers()
                     .auth_bearer(&self.config.token)
                     .json(PostData::new(&content, &images, &flags));

        let req = match req {
            Ok(req) => req,
            Err(error) => return Box::new(future::err(format!("Gab post actix error: {}", error)))
        };

        let req = req.send_ext().map_err(|error| format!("Gab post error: {}", error))
                     .and_then(|response| match response.status().is_success() {
                         true => Ok(response),
                         false => Err(format!("Gab server returned error code {}", response.status())),
                     })
                     .and_then(|response| response.json::<PostResponse>().map_err(|error| format!("Gab post error: {}", error)))
                     .map(|response| ResultMessage::Id(response.post.id));

        Box::new(req)
    }
}
