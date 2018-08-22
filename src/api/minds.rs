use config;
use data::PostFlags;
use http::{multipart, AutoClient, AutoRuntime, IntoFuture, Future, Mime, Request};
use ::http;

const OAUTH2_URL: &'static str = "https://www.minds.com/oauth2/token";
const IMAGES_URL: &'static str = "https://www.minds.com/api/v1/media";
const POST_URL: &'static str = "https://www.minds.com/api/v1/newsfeed";

pub struct Minds {
    token: String,
}

impl Minds {
    pub fn new(config: config::Minds) -> Option<Self> {
        let req = Request::post(OAUTH2_URL)
            .expect("To create request")
            .json(&Auth::new(&config.username, &config.password))
            .expect("To serialize json")
            .send()
            .finish();

        let req = match req {
            Ok(req) => req,
            Err(error) => {
                eprintln!("Minds: Failed to authorize. Error: {:?}", error);
                return None;
            },
        };

        if !req.is_success() {
            eprintln!("Minds: Failed to authorize. Status code={}", req.status());
            return None;
        }

        let oauth2 = match req.json::<Oauth2>().finish() {
            Ok(oauth2) => oauth2,
            Err(error) => {
                eprintln!("Minds: Invalid response to authorize. Error: {:?}", error);
                return None;
            },
        };

        Some(Self { token: oauth2.access_token })
    }

    pub fn upload_image(&self, name: &str, mime: &Mime, data: &[u8]) -> impl Future<Item = String, Error = ()> {
        let mut form = multipart::Form::new();
        form.add_file_field("file".to_string(), name.to_string(), mime, data);

        let req = Request::post(IMAGES_URL).expect("To create request").bearer_auth(&self.token).multipart(form).send();

        req.map_err(|error| eprintln!("Minds: uploading image Error={:?}", error))
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => {
                    eprintln!("Minds: failed to upload image. Status code={}", resp.status());
                    Err(())
                },
            }).and_then(|response| response.json::<UploadResponse>().map_err(|error| eprintln!("Minds upload reading error: {:?}", error)))
            .map(|response| response.guid)
    }

    pub fn post(&self, message: &str, media_attachments: Option<String>, flags: &PostFlags) -> impl Future<Item = (), Error = ()> {
        let req = Request::post(POST_URL)
            .expect("To create request")
            .bearer_auth(self.token.as_str())
            .json(&Post::new(&message, &media_attachments, &flags))
            .expect("To serialzie post data")
            .send();

        //For image we wait twice of time
        //just to be sure
        req.or_else(|resp| resp.retry(http::get_timeout()).into_future().flatten())
           .map_err(|error| eprintln!("Minds: post error. Error={:?}", error))
           .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => {
                    eprintln!("Minds: failed to post. Status code={}", resp.status());
                    Err(())
                },
            }).and_then(|resp| resp.json::<UploadResponse>().map_err(|error| eprintln!("Minds: Invalid response. Error={:?}", error)))
           .map(|resp| println!("Minds(id={}) OK", resp.guid))
    }
}

#[derive(Serialize, Debug)]
struct Auth<'a> {
    grant_type: &'static str,
    client_id: &'static str,
    client_secret: &'static str,
    username: &'a str,
    password: &'a str,
}

impl<'a> Auth<'a> {
    fn new(username: &'a str, password: &'a str) -> Self {
        Auth {
            grant_type: "password",
            client_id: "",
            client_secret: "",
            username,
            password,
        }
    }
}

#[derive(Deserialize, Debug)]
struct Oauth2 {
    pub access_token: String,
    pub user_id: String,
    pub refresh_token: String,
}

#[derive(Serialize, Debug)]
struct Post<'a> {
    wire_threshold: Option<String>,
    message: &'a str,
    is_rich: u8,
    title: Option<String>,
    description: Option<String>,
    thumbnail: Option<String>,
    url: Option<String>,
    attachment_guid: &'a Option<String>,
    pub mature: u8,
    access_id: u8,
}

impl<'a> Post<'a> {
    fn new(message: &'a str, attachment_guid: &'a Option<String>, flags: &PostFlags) -> Self {
        Post {
            wire_threshold: None,
            message,
            is_rich: 0,
            title: None,
            description: None,
            thumbnail: None,
            url: None,
            attachment_guid,
            mature: flags.nsfw as u8,
            access_id: 2,
        }
    }
}

#[derive(Deserialize, Debug)]
struct UploadResponse {
    pub guid: String,
}
