use ::futures::{future, Stream};
use ::tokio_core::reactor::{
    Core
};
use ::serde_json;

use ::config;
use ::utils::{
    empty_future_job,
    Image
};
use super::http;
use self::http::{
    MultipartBody,
};

const OAUTH2_URL: &'static str = "https://www.minds.com/oauth2/token";
const POST_URL: &'static str = "https://www.minds.com/api/v1/newsfeed";
const IMAGES_URL: &'static str = "https://www.minds.com/api/v1/media";

pub mod payload {
    #[derive(Serialize, Debug)]
    pub struct Auth {
        grant_type: &'static str,
        client_id: &'static str,
        client_secret: &'static str,
        username: String,
        password: String,
    }

    impl Auth {
        pub fn new(username: String, password: String) -> Self {
            Auth {
                grant_type: "password",
                client_id: "",
                client_secret: "",
                username,
                password
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Oauth2 {
        pub access_token: String,
        pub user_id: String,
        pub refresh_token: String
    }

    #[derive(Serialize, Debug)]
    pub struct Post<'a> {
        wire_threshold: Option<String>,
        message: &'a str,
        is_rich: u8,
        title: Option<String>,
        description: Option<String>,
        thumbnail: Option<String>,
        url: Option<String>,
        attachment_guid: Option<String>,
        mature: u8,
        access_id: u8
    }

    impl<'a> Post<'a> {
        pub fn new(message: &'a str, attachment_guid: Option<String>) -> Self {
            Post {
                wire_threshold: None,
                message,
                is_rich: 0,
                title: None,
                description: None,
                thumbnail: None,
                url: None,
                attachment_guid,
                mature: 0,
                access_id: 2
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct UploadResponse {
        pub guid: String
    }
}

pub struct Client<'a> {
    http: &'a http::HttpClient,
    oauth2: payload::Oauth2
}

impl<'a> Client<'a> {
    ///Creates new instance of client and performs authorization.
    pub fn new(http: &'a http::HttpClient, core: &mut Core, config: config::Minds) -> Result<Self, String> {
        let oauth2 = {
            let mut req = http::Request::new(http::Method::Post, OAUTH2_URL.parse().unwrap());
            req.headers_mut().set(http::Accept::json());
            req.headers_mut().set(http::ContentType::json());
            let auth_body = payload::Auth::new(config.username, config.password);
            req.set_body(serde_json::to_string(&auth_body).unwrap());
            let response = core.run(http.request(req)).map_err(error_formatter!("Minds: Cannot send auth request."))?;
            let body = core.run(response.body().concat2()).map_err(error_formatter!("Minds: Cannot read oauth2's response."))?;
            serde_json::from_slice(&body).map_err(error_formatter!("Minds: Cannot parse oauth22's response."))?
        };

        Ok(Client {
            http,
            oauth2
        })
    }

    fn auth(&self) -> http::Authorization<http::Bearer> {
        http::Authorization(http::Bearer {
            token: self.oauth2.access_token.clone()
        })
    }

    ///Uploads image
    ///
    ///NOTE: Minds.com allows only one attachment
    pub fn upload_image(&self, image: &Image) -> http::FutureResponse {
        let mut req = http::Request::new(http::Method::Post, IMAGES_URL.parse().unwrap());
        req.headers_mut().set(self.auth());
        req.set_multipart_body("-fie", &image.name, &image.mime, &image.content);

        self.http.request(req)
    }

    ///Post new message.
    pub fn post(&self, message: &str, attachment_guid: Option<String>) -> http::FutureResponse {
        let message = payload::Post::new(message, attachment_guid);

        let mut req = http::Request::new(http::Method::Post, POST_URL.parse().unwrap());
        req.headers_mut().set(http::ContentType::json());
        req.headers_mut().set(self.auth());
        req.set_body(serde_json::to_string(&message).unwrap());

        self.http.request(req)
    }

    pub fn handle_post(result: Result<http::Response, String>) -> future::FutureResult<(), ()> {
        println!(">>>Minds:");
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
