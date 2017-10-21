mod hyper {
    pub use ::hyper::{Client, Request, Method, Response, StatusCode};
    pub use ::hyper::header::{ContentType, ContentLength, Authorization, Bearer};
    pub use ::hyper::client::{HttpConnector, FutureResponse};
    pub use ::hyper_tls::{HttpsConnector};
}

use ::serde_json;
use ::tokio_core::reactor::{
    Handle
};

use super::common;
use ::config;

const POST_URL: &'static str = "https://gab.ai/posts";

mod payload {
    #[derive(Serialize, Debug)]
    pub struct Post {
        body: String,
        reply_to: String,
        is_quote: u8,
        gif: String,
        category: Option<String>,
        topic: Option<String>,
        share_twitter: bool,
        share_facebook: bool,
        is_replies_disabled: bool
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
                is_replies_disabled: false
            }
        }
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

    pub fn handle_post(response: hyper::Response) -> Result<(), String> {
        if response.status() != hyper::StatusCode::Ok {
            return Err(format!("Failed to post. Status: {}", response.status()));
        }

        println!("OK");
        Ok(())
    }
}
