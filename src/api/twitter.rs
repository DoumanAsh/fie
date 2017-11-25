//!Twitter accessing module
use ::egg_mode::{
    Token,
    KeyPair,
    FutureResponse,
    Response,
    tweet,
    media
};
use ::futures::future;
use ::tokio_core::reactor::{
    Handle
};

use ::config;

use ::utils::{
    empty_future_job,
    Image
};

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
    pub fn upload_image<'a>(&self, image: &'a Image) -> media::UploadFuture<'a> {
        media::UploadBuilder::new(&image.content[..], image.mime.clone()).call(&self.token, &self.handle)
    }

    ///Posts new tweet.
    pub fn post(&self, message: &str, images: &[u64]) -> FutureResponse<tweet::Tweet> {
        tweet::DraftTweet::new(message).media_ids(images).send(&self.token, &self.handle)
    }

    pub fn handle_post(result: Result<Response<tweet::Tweet>, String>) -> future::FutureResult<(), ()> {
        println!(">>>Twitter:");
        match result {
            Ok(rsp) => println!("OK(id={})", rsp.response.id),
            Err(error) => println!("{}", error)
        }

        empty_future_job()
    }
}
