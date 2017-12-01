use ::futures::future;
use ::serde_json;

use super::http;
use self::http::{
    MultipartBody,
};

use ::config;
use ::utils::{
    empty_future_job,
    Image
};

const POST_URL: &'static str = "https://gab.ai/posts";
const IMAGES_URL: &'static str = "https://gab.ai/api/media-attachments/images";

pub mod payload {
    #[derive(Serialize, Debug)]
    pub struct Post<'a> {
        body: &'a str,
        pub reply_to: &'a str,
        pub is_quote: u8,
        pub nsfw: u8,
        pub is_premium: u8,
        pub _method: &'a str,
        pub gif: &'a str,
        pub category: Option<&'a str>,
        pub topic: Option<&'a str>,
        pub share_twitter: bool,
        pub share_facebook: bool,
        pub is_replies_disabled: bool,
        pub media_attachments: &'a [String]
    }

    impl<'a> Post<'a> {
        pub fn new(message: &'a str, media_attachments: &'a [String]) -> Self {
            Post {
                body: message,
                reply_to: "",
                is_quote: 0,
                nsfw: 0,
                is_premium: 0,
                _method: "post",
                gif: "",
                category: None,
                topic: None,
                share_twitter: false,
                share_facebook: false,
                is_replies_disabled: false,
                media_attachments
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct UploadResponse {
        pub id: String
    }
}

///Gab.ai Client
pub struct Client<'a> {
    http: &'a http::HttpClient,
    config: config::Gab
}

impl<'a> Client<'a> {
    ///Creates new instance of client and performs authorization.
    pub fn new(http: &'a http::HttpClient, config: config::Gab) -> Self {
        Client {
            http,
            config
        }
    }

    fn auth(&self) -> http::Authorization<http::Bearer> {
        http::Authorization(http::Bearer {
            token: self.config.token.clone()
        })
    }

    ///Uploads image to gab.ai.
    pub fn upload_image(&self, image: &Image) -> http::FutureResponse {
        let mut req = http::Request::new(http::Method::Post, IMAGES_URL.parse().unwrap());
        req.headers_mut().set(self.auth());
        req.set_multipart_body("-fie", &image.name, &image.mime, &image.content);

        self.http.request(req)
    }

    ///Posts new message.
    pub fn post(&self, message: &str, images: &[String]) -> http::FutureResponse {
        let message = payload::Post::new(message, images);

        let mut req = http::Request::new(http::Method::Post, POST_URL.parse().unwrap());
        req.headers_mut().set(http::ContentType::json());
        req.headers_mut().set(self.auth());
        req.set_body(serde_json::to_string(&message).unwrap());

        self.http.request(req)
    }

    pub fn handle_post(result: Result<http::Response, String>) -> future::FutureResult<(), ()> {
        println!(">>>Gab:");
        match result {
            Ok(response) => {
                if response.status() != http::StatusCode::Ok {
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
