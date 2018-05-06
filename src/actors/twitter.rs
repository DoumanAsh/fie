//! Actors to access twitter API

extern crate futures;
extern crate egg_mode;
extern crate actix;

use self::futures::Future;
use self::actix::prelude::*;
use self::egg_mode::{
    Token,
    KeyPair,
    media,
    tweet
};

use ::config;
use super::messages::{
    UploadImage,
    ResultImage,
    PostMessage,
    ResultMessage
};

///Twitter actor
pub struct Twitter {
    token: Token
}

impl Actor for Twitter {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
    }
}

impl Twitter {
    pub fn new(config: config::Twitter) -> Self {
        let token = Token::Access {
            consumer: KeyPair::new(config.consumer.key, config.consumer.secret),
            access: KeyPair::new(config.access.key, config.access.secret)
        };

        Self {
            token
        }
    }
}

impl Handler<UploadImage> for Twitter {
    type Result = ResponseFuture<ResultImage, String>;

    fn handle(&mut self, msg: UploadImage, _: &mut Self::Context) -> Self::Result {
        let mime = &msg.0.mime;
        let data = msg.0.mmap.to_vec();

        let result = media::UploadBuilder::new(data, mime.clone()).call(&self.token, Arbiter::handle());
        let result = result.map(|result| ResultImage::Id(result.id))
                           .map_err(|error| format!("Twitter image upload error: {}", error));

        Box::new(result)
    }
}

impl Handler<PostMessage> for Twitter {
    type Result = ResponseFuture<ResultMessage, String>;

    fn handle(&mut self, msg: PostMessage, _: &mut Self::Context) -> Self::Result {
        let PostMessage{flags, content, images} = msg;

        let result = tweet::DraftTweet::new(content).possibly_sensitive(flags.nsfw);
        let result = match images {
            Some(mut images) => {
                let images: Vec<u64> = images.drain(..).map(|image| image.id()).collect();
                result.media_ids(&images[..])
            },
            None => result,
        };
        let result = result.send(&self.token, Arbiter::handle())
                           .map(|result| ResultMessage::Id(result.id))
                           .map_err(|error| format!("Twitter failed to post: {}", error));

        Box::new(result)
    }
}
