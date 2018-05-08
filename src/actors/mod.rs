//! Fie's actors
extern crate actix;
extern crate futures;

use std::rc::Rc;

use self::actix::prelude::*;
use self::futures::{future, Future};

pub mod gab;
pub mod messages;
pub mod minds;
pub mod twitter;

pub use self::gab::Gab;
pub use self::minds::Minds;
pub use self::twitter::Twitter;

use cli::Post;
use config;
use io;

/// United API Actor
pub struct API {
    pub twitter: Option<Addr<Unsync, Twitter>>,
    pub gab: Option<Addr<Unsync, Gab>>,
    pub minds: Option<Addr<Unsync, Minds>>,
}

impl API {
    pub fn new() -> Self {
        Self {
            twitter: None,
            gab: None,
            minds: None,
        }
    }

    pub fn start_minds_if(mut self, cond: bool, minds: config::Minds) -> Self {
        if cond {
            self.minds = Some(Minds::new(minds).start());
        }

        self
    }

    pub fn start_gab_if(mut self, cond: bool, gab: config::Gab) -> Self {
        if cond {
            self.gab = Some(Gab::new(gab).start());
        }

        self
    }

    pub fn start_twitter_if(mut self, cond: bool, twitter: config::Twitter) -> Self {
        if cond {
            self.twitter = Some(Twitter::new(twitter).start());
        }

        self
    }

    pub fn join_hash_tags(tags: Vec<String>) -> String {
        let mut result = String::new();

        for tag in tags {
            result.push_str(&format!("#{} ", tag));
        }

        // remove last white space
        let _ = result.pop();

        result
    }
}

impl Actor for API {
    type Context = Context<Self>;
}

impl Message for Post {
    type Result = Result<(), ()>;
}

impl Handler<Post> for API {
    type Result = ResponseFuture<(), ()>;

    fn handle(&mut self, msg: Post, ctx: &mut Self::Context) -> Self::Result {
        let Post { message, tags, flags, images } = msg;

        let message = if tags.len() > 0 {
            match message.as_str() {
                "" => Self::join_hash_tags(tags),
                message => format!("{}\n{}", message, Self::join_hash_tags(tags)),
            }
        } else {
            message
        };

        // this is post base's part.
        // as each API accepts images in own format.
        // we'll need set accordingly
        let post = messages::PostMessage {
            flags,
            content: message,
            images: None,
        };

        match images {
            Some(ref images) if images.len() > 0 => {
                let images = {
                    let mut result = vec![];
                    for image in images {
                        match io::Image::open(image) {
                            Ok(image) => result.push(Rc::new(image)),
                            Err(error) => {
                                eprintln!("Error opening image '{}': {}", image, error);
                                return Box::new(future::ok(()));
                            },
                        };
                    }
                    result
                };

                let mut jobs: Vec<ResponseFuture<(), ()>> = vec![];

                if let Some(twitter) = self.twitter.take() {
                    if twitter.connected() {
                        // If not connected do not return it back
                        let mut tweet_images: Vec<_> = vec![];
                        for image in images.iter() {
                            let upload_img = messages::UploadImage(image.clone());
                            let upload_img = twitter.send(upload_img).map_err(|error| format!("Tweet upload img actix mailbox error: {}", error));
                            tweet_images.push(upload_img)
                        }

                        let self_addr: Addr<Unsync, _> = ctx.address();

                        let mut post = post.clone();
                        let tweet_upload = future::join_all(tweet_images).map_err(|error| eprintln!("{}", error));
                        let tweet_upload = tweet_upload.and_then(move |result| -> ResponseFuture<(), ()> {
                            let mut tweet_images = vec![];
                            for res in result {
                                match res {
                                    Ok(image) => tweet_images.push(image),
                                    Err(error) => {
                                        eprintln!("{}", error);
                                        return Box::new(future::ok(()));
                                    },
                                }
                            }
                            post.images = Some(tweet_images);

                            let result = self_addr
                                .send(PostTweet(post))
                                .map_err(|error| {
                                    eprintln!("Tweet upload mailbox error: {}", error);
                                })
                                .map(|_| ());

                            Box::new(result)
                        });

                        // to guarantee that future's result will not be error
                        jobs.push(Box::new(tweet_upload.or_else(|_| Ok(()))));
                        self.twitter = Some(twitter);
                    }
                }
                if let Some(minds) = self.minds.take() {
                    if minds.connected() {
                        // TODO: For now Minds.com accepts only one attachment.
                        match images.len() {
                            0 => {
                                eprintln!("Unexpected error. Minds actor gets 0 images but should be at least one");
                                return Box::new(future::err(()));
                            },
                            1 => (),
                            _ => eprintln!("Minds.com accepts only one attachment, only first image will be attached"),
                        }

                        let self_addr: Addr<Unsync, _> = ctx.address();

                        let mut post = post.clone();
                        let image = unsafe { images.get_unchecked(0).clone() };
                        let image = messages::UploadImage(image);
                        let minds_post = minds.send(image).map_err(|error| eprintln!("Minds upload img actix mailbox error: {}", error)).and_then(
                            move |result| -> ResponseFuture<(), ()> {
                                match result {
                                    Err(error) => {
                                        eprintln!("{}", error);
                                        return Box::new(future::ok(()));
                                    },
                                    Ok(image) => {
                                        post.images = Some(vec![image]);

                                        let result = self_addr
                                            .send(PostMinds(post))
                                            .map_err(|error| {
                                                eprintln!("Minds upload mailbox error: {}", error);
                                            })
                                            .map(|_| ());

                                        Box::new(result)
                                    },
                                }
                            },
                        );

                        jobs.push(Box::new(minds_post.or_else(|_| Ok(()))));
                        self.minds = Some(minds);
                    }
                }
                if let Some(gab) = self.gab.take() {
                    if gab.connected() {
                        let mut gab_images: Vec<_> = vec![];
                        for image in images.iter() {
                            let upload_img = messages::UploadImage(image.clone());
                            let upload_img = gab.send(upload_img).map_err(|error| format!("Gab upload img actix mailbox error: {}", error));
                            gab_images.push(upload_img)
                        }

                        let self_addr: Addr<Unsync, _> = ctx.address();

                        let mut post = post;
                        let gab_upload = future::join_all(gab_images).map_err(|error| eprintln!("{}", error));
                        let gab_upload = gab_upload.and_then(move |result| -> ResponseFuture<(), ()> {
                            let mut gab_images = vec![];
                            for res in result {
                                match res {
                                    Ok(image) => gab_images.push(image),
                                    Err(error) => {
                                        eprintln!("{}", error);
                                        return Box::new(future::ok(()));
                                    },
                                }
                            }
                            post.images = Some(gab_images);

                            let result = self_addr
                                .send(PostGab(post))
                                .map_err(|error| {
                                    eprintln!("Gab upload mailbox error: {}", error);
                                })
                                .map(|_| ());

                            Box::new(result)
                        });

                        jobs.push(Box::new(gab_upload.or_else(|_| Ok(()))));
                        self.gab = Some(gab);
                    }
                }

                Box::new(future::join_all(jobs).map(|_| ()))
            },
            _ => {
                let mut jobs: Vec<_> = vec![];

                if self.twitter.is_some() {
                    jobs.push(self.handle(PostTweet(post.clone()), ctx));
                }
                if self.gab.is_some() {
                    jobs.push(self.handle(PostGab(post.clone()), ctx));
                }
                if self.minds.is_some() {
                    jobs.push(self.handle(PostMinds(post), ctx));
                }

                Box::new(future::join_all(jobs).map(|_| ()))
            },
        }
    }
}

/// Performs post on twatter
pub struct PostTweet(messages::PostMessage);
impl Message for PostTweet {
    type Result = Result<(), ()>;
}

impl Handler<PostTweet> for API {
    type Result = ResponseFuture<(), ()>;

    fn handle(&mut self, msg: PostTweet, _ctx: &mut Self::Context) -> Self::Result {
        let msg = msg.0;

        if let Some(twitter) = self.twitter.take() {
            if !twitter.connected() {
                return Box::new(future::ok(()));
            }

            let tweet = twitter
                .send(msg)
                .map(|result| match result {
                    Ok(result) => println!("Tweet(id={}) OK", result),
                    Err(error) => println!("Tweet Error: {}", error),
                })
                .or_else(|error| {
                    eprintln!("Twitter Actix mailbox error: {}", error);
                    Ok(())
                });
            self.twitter = Some(twitter);
            Box::new(tweet)
        } else {
            Box::new(future::ok(()))
        }
    }
}

/// Performs post on gab
pub struct PostGab(messages::PostMessage);
impl Message for PostGab {
    type Result = Result<(), ()>;
}

impl Handler<PostGab> for API {
    type Result = ResponseFuture<(), ()>;

    fn handle(&mut self, msg: PostGab, _ctx: &mut Self::Context) -> Self::Result {
        let msg = msg.0;
        if let Some(gab) = self.gab.take() {
            if !gab.connected() {
                return Box::new(future::ok(()));
            }

            let job = gab.send(msg)
                .map(|result| match result {
                    Ok(result) => println!("Gab(id={}) OK", result),
                    Err(error) => println!("Gab Error: {}", error),
                })
                .or_else(|error| {
                    eprintln!("Gab Actix mailbox error: {}", error);
                    Ok(())
                });
            self.gab = Some(gab);
            Box::new(job)
        } else {
            Box::new(future::ok(()))
        }
    }
}

/// Performs post on minds
pub struct PostMinds(messages::PostMessage);
impl Message for PostMinds {
    type Result = Result<(), ()>;
}

impl Handler<PostMinds> for API {
    type Result = ResponseFuture<(), ()>;

    fn handle(&mut self, msg: PostMinds, _ctx: &mut Self::Context) -> Self::Result {
        let msg = msg.0;
        if let Some(minds) = self.minds.take() {
            if !minds.connected() {
                return Box::new(future::ok(()));
            }

            let job = minds
                .send(msg)
                .map(|result| match result {
                    Ok(result) => println!("Minds(id={}) OK", result),
                    Err(error) => println!("Minds Error: {}", error),
                })
                .or_else(|error| {
                    eprintln!("Minds Actix mailbox error: {}", error);
                    Ok(())
                });
            self.minds = Some(minds);
            Box::new(job)
        } else {
            Box::new(future::ok(()))
        }
    }
}
