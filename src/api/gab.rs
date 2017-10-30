mod hyper {
    pub use ::hyper::{Client, Request, Method, Response, StatusCode};
    pub use ::hyper::header::{ContentType, ContentLength, Authorization, Bearer};
    pub use ::hyper::client::{HttpConnector, FutureResponse};
    pub use ::hyper_tls::{HttpsConnector};
}
use ::futures::future;
use ::hyper::mime;

use ::serde_json;
use ::tokio_core::reactor::{
    Handle
};

use super::common;
use ::config;
use ::utils::{
    empty_future_job,
    Image
};

const POST_URL: &'static str = "https://gab.ai/posts";
const IMAGES_URL: &'static str = "https://gab.ai/api/media-attachments/images";

pub mod payload {
    #[derive(Serialize, Debug)]
    pub struct Post {
        body: String,
        pub reply_to: String,
        pub is_quote: u8,
        pub gif: String,
        pub category: Option<String>,
        pub topic: Option<String>,
        pub share_twitter: bool,
        pub share_facebook: bool,
        pub is_replies_disabled: bool,
        pub media_attachments: Vec<String>
    }

    impl Post {
        pub fn new(message: String) -> Self {
            Post {
                body: message,
                reply_to: "".to_string(),
                is_quote: 0,
                gif: "".to_string(),
                category: None,
                topic: None,
                share_twitter: false,
                share_facebook: false,
                is_replies_disabled: false,
                media_attachments: Vec::new()
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct UploadResponse {
        pub id: String
    }
}

///Gab.ai Client
pub struct Client {
    hyper: hyper::Client<hyper::HttpsConnector<hyper::HttpConnector>>,
    config: config::Gab
}

impl Client {
    ///Creates new instance of client and performs authorization.
    pub fn new(handle: Handle, config: config::Gab) -> Self {
        let hyper = hyper::Client::configure().keep_alive(true)
                                              .connector(hyper::HttpsConnector::new(4, &handle).unwrap())
                                              .build(&handle);

        Client {
            hyper,
            config
        }
    }

    fn auth(&self) -> hyper::Authorization<hyper::Bearer> {
        hyper::Authorization(hyper::Bearer {
            token: self.config.token.clone()
        })
    }

    fn multipart_mime() -> mime::Mime {
        "multipart/form-data; boundary=-fie".parse().unwrap()
    }

    fn multipart_body(image: &Image) -> (Vec<u8>, u64) {
        let mut body = Vec::with_capacity(image.content.len());
        body.extend("\r\n---fie\r\n".as_bytes().iter());
        body.extend(format!("Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n", image.name).as_bytes().iter());
        body.extend(format!("Content-Type: {}\r\n\r\n", image.mime).as_bytes().iter());
        body.extend(image.content.iter());
        body.extend("\r\n---fie--\r\n".as_bytes().iter());
        let len = body.len() as u64;
        (body, len)
    }

    ///Uploads image to gab.ai.
    pub fn upload_image(&self, image: &Image) -> hyper::FutureResponse {
        let mut req = hyper::Request::new(hyper::Method::Post, IMAGES_URL.parse().unwrap());
        req.headers_mut().set(hyper::ContentType(Self::multipart_mime()));
        req.headers_mut().set(self.auth());

        let (payload, len) = Self::multipart_body(image);
        req.headers_mut().set(hyper::ContentLength(len));
        req.set_body(payload);

        self.hyper.request(req)
    }

    ///Post new message.
    pub fn post(&self, message: &str, tags: &Option<Vec<String>>) -> hyper::FutureResponse {
        let message = common::message(message, tags);
        let message = payload::Post::new(message);

        let mut req = hyper::Request::new(hyper::Method::Post, POST_URL.parse().unwrap());
        req.headers_mut().set(hyper::ContentType::json());
        req.headers_mut().set(self.auth());
        req.set_body(serde_json::to_string(&message).unwrap());

        self.hyper.request(req)
    }

    ///Posts new message with image
    pub fn post_w_images(&self, message: &str, tags: &Option<Vec<String>>, images: &[String]) -> hyper::FutureResponse {
        let message = common::message(message, tags);
        let mut message = payload::Post::new(message);
        message.media_attachments.extend(images.iter().cloned());

        let mut req = hyper::Request::new(hyper::Method::Post, POST_URL.parse().unwrap());
        req.headers_mut().set(hyper::ContentType::json());
        req.headers_mut().set(self.auth());
        req.set_body(serde_json::to_string(&message).unwrap());

        self.hyper.request(req)
    }

    pub fn handle_post(result: Result<hyper::Response, String>) -> future::FutureResult<(), ()> {
        println!(">>>Gab:");
        match result {
            Ok(response) => {
                if response.status() != hyper::StatusCode::Ok {
                    println!("Failed to post. Status: {}", response.status());
                }
                else {
                    println!("OK");
                }
            }
            Err(error) => println!("{}", error)
        }

        empty_future_job()
    }
}
