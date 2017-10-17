//!Twitter accessing module
use ::egg_mode::{
    Token,
    KeyPair,
    FutureResponse,
    tweet,
};

use ::tokio_core::reactor::{
    Handle
};

pub const ACCESS_TOKEN_ENV: &'static str = "TWITTER_ACCESS_TOKEN";
const CONSUMER_TOKEN: &'static str = include_str!("../../tw_consumer.token");

///Twitter client.
pub struct Client {
    ///Twitter access token.
    token: Token,
    ///Tokio Core's handle
    handle: Handle
}

impl Client {
    ///Creates new instances and initializes token.
    pub fn new(handle: Handle, access_key: &str, access_secret: &str) -> Self {
        let mut consumer_split = CONSUMER_TOKEN.split_whitespace();
        let consumer_key = consumer_split.next().expect("Consumer key is missing from tw_consumer.token file!");
        let consumer_secret = consumer_split.next().expect("Consumer secret is missing from tw_consumer.token file!");

        let token = Token::Access {
            consumer: KeyPair::new(consumer_key, consumer_secret),
            access: KeyPair::new(access_key.to_string(), access_secret.to_string())
        };

        Client {
            token,
            handle
        }
    }

    ///Posts new tweet.
    pub fn post(&self, message: String, tags: Option<Vec<String>>) -> FutureResponse<tweet::Tweet> {
        let message = match tags {
            Some(tags) => format!("{}\n{}", message, tags.join(" ")),
            None => message
        };

        tweet::DraftTweet::new(&message).send(&self.token, &self.handle)
    }
}
