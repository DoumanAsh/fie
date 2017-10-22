//!Twitter accessing module
use ::egg_mode::{
    Token,
    KeyPair,
    FutureResponse,
    Response,
    tweet,
    media
};

use ::tokio_core::reactor::{
    Handle
};

use super::common;
use ::config;

use ::utils::Image;

///Twitter client.
pub struct Client {
    ///Twitter access token.
    token: Token,
    ///Tokio Core's handle
    handle: Handle
}

impl Client {
    ///Creates new instances and initializes token.
    pub fn new(handle: Handle, config: config::Twitter) -> Self {
        let token = Token::Access {
            consumer: KeyPair::new(config.consumer.key, config.consumer.secret),
            access: KeyPair::new(config.access.key, config.access.secret)
        };

        Client {
            token,
            handle
        }
    }

    ///Uploads image to twitter.
    pub fn upload_image(&self, image: &Image) -> FutureResponse<media::Media> {
        media::upload_image(&image.content, &self.token, &self.handle)
    }

    ///Posts new tweet.
    pub fn post(&self, message: &str, tags: &Option<Vec<String>>) -> FutureResponse<tweet::Tweet> {
        let message = common::message(message, tags);

        tweet::DraftTweet::new(&message).send(&self.token, &self.handle)
    }

    ///Posts new tweet with images.
    pub fn post_w_images(&self, message: &str, tags: &Option<Vec<String>>, images: &[u64]) -> FutureResponse<tweet::Tweet> {
        let message = common::message(message, tags);

        tweet::DraftTweet::new(&message).media_ids(images).send(&self.token, &self.handle)
    }

    pub fn handle_post(response: Response<tweet::Tweet>) -> Result<(), String> {
        Ok(println!("OK(id={})", response.response.id))
    }
}
