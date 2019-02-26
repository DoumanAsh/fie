use crate::config;
use crate::data::PostFlags;
use crate::http::{self, Uri, multipart, Future, IntoFuture, Mime, Request, AutoClient};

#[derive(Deserialize, Debug)]
pub struct EntityId {
    id: String
}

#[derive(Serialize, Debug)]
pub struct NewStatus<'a> {
    status: &'a str,
    pub media_ids: &'a [String],
    sensitive: bool,
}

impl<'a> NewStatus<'a> {
    pub fn new(status: &'a str, media_ids: &'a [String], flags: &PostFlags) -> Self {
        Self {
            status,
            media_ids,
            sensitive: flags.nsfw,
        }
    }
}

pub struct Mastodon {
    config: config::Mastodon,
}

impl Mastodon {
    pub fn new(config: config::Mastodon) -> Option<Self> {
        match config.host.parse::<Uri>() {
            Ok(_) => (),
            Err(_) => {
                eprintln!("Mastodon: Invalid host URI {}", config.host);
                return None
            }
        }

        if config.access_token.len() == 0 {
            eprintln!("Mastodon: Empty access token, cannot use Mastodon");
            return None
        }

        Some(Self {
            config
        })
    }

    pub fn upload_image(&self, name: &str, mime: &Mime, data: &[u8]) -> impl Future<Item = String, Error = ()> {
        let url = format!("https://{}/api/v1/media", &self.config.host);
        let mut form = multipart::Form::new();

        form.add_file_field("file".to_string(), name.to_string(), mime, data);

        let req = Request::post(url).expect("To create request").bearer_auth(self.config.access_token.as_str()).multipart(form).send();

        // For image we wait twice of time
        // just to be sure
        req.or_else(|resp| resp.retry(http::get_timeout()).into_future().flatten())
            .map_err(|error| eprintln!("Mastodon: uploading image Error={}", error))
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => {
                    eprintln!("Mastodon: failed to upload image. Status code={}", resp.status());
                    Err(())
                },
            }).and_then(|response| response.json::<EntityId>().map_err(|error| eprintln!("Mastodon upload reading error: {}", error)))
            .map(|response| response.id)

    }

    pub fn post(&self, message: &str, media_attachments: &[String], flags: &PostFlags) -> impl Future<Item = (), Error = ()> {
        let url = format!("https://{}/api/v1/statuses", &self.config.host);
        let req = Request::post(url).expect("To create request")
                                    .bearer_auth(self.config.access_token.as_str())
                                    .json(&NewStatus::new(&message, &media_attachments, &flags))
                                    .expect("To serialzie post data")
                                    .send();

        req.map_err(|error| eprintln!("Mastodon: post error. Error={}", error))
            .and_then(|resp| match resp.is_success() {
                true => Ok(resp),
                false => {
                    eprintln!("Mastodon: failed to post. Status code={}", resp.status());
                    Err(())
                },
            }).and_then(|resp| resp.json::<EntityId>().map_err(|error| eprintln!("Mastodon: Invalid response. Error={}", error)))
            .map(|resp| println!("Mastodon(id={}) OK", resp.id))

    }
}
