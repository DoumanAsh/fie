//!Twitter accessing module
use ::egg_mode::{
    Token,
    KeyPair,
    WebResponse,
    tweet,
};

pub const ACCESS_TOKEN_ENV: &'static str = "TWITTER_ACCESS_TOKEN";
const CONSUMER_TOKEN: &'static str = include_str!("../../tw_consumer.token");

///Creates Twitter token
pub fn create_token<'a>(access_key: &'a str, access_secret: &'a str) -> Token<'a> {
    let mut consumer_split = CONSUMER_TOKEN.split_whitespace();
    let consumer_key = consumer_split.next().expect("Consumer key is missing from tw_consumer.token file!");
    let consumer_secret = consumer_split.next().expect("Consumer secret is missing from tw_consumer.token file!");

    Token::Access {
        consumer: KeyPair::new(consumer_key, consumer_secret),
        access: KeyPair::new(access_key, access_secret)
    }
}

pub fn post_tweet(message: String, tags: Option<Vec<String>>, token: &Token) -> WebResponse<tweet::Tweet> {
    let message = match tags {
        Some(tags) => format!("{}\n{}", message, tags.join(" ")),
        None => message
    };

    tweet::DraftTweet::new(&message).send(token)
}
