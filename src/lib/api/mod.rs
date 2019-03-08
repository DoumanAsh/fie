//!Social medias API module

mod http;
pub mod mastodon;
pub mod gab;
pub mod twitter;

use twitter::{Twitter, TwitterError};
use gab::{Gab, GabError};
use mastodon::{Mastodon, MastodonError};
use http::{future, Future, AutoRuntime, HttpRuntime};
use crate::data::{join_hash_tags, PostId, Post};

use super::config;

use std::fmt;
use std::error::Error;
use std::io;

#[derive(Debug)]
///API Errors
pub enum ApiError {
    ///Unable to load Image for attachment
    CannotLoadImage(String, io::Error),
    ///Mastodon error
    Mastodon(MastodonError),
    ///Gab error
    Gab(GabError),
    ///Twitter error
    Twitter(TwitterError),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ApiError::CannotLoadImage(ref name, ref error) => write!(f, "Error opening image '{}'. Error: {}", name, error),
            &ApiError::Mastodon(ref error) => write!(f, "Mastodon API Error: {}", error),
            &ApiError::Gab(ref error) => write!(f, "Gab API Error: {}", error),
            &ApiError::Twitter(ref error) => write!(f, "Twitter API Error: {}", error),
        }
    }
}

impl Error for ApiError {}

impl From<MastodonError> for ApiError {
    fn from(error: MastodonError) -> Self {
        ApiError::Mastodon(error)
    }
}

impl From<GabError> for ApiError {
    fn from(error: GabError) -> Self {
        ApiError::Gab(error)
    }
}

impl From<TwitterError> for ApiError {
    fn from(error: TwitterError) -> Self {
        ApiError::Twitter(error)
    }
}


type PostResultInner = (Option<Result<PostId, ApiError>>, Option<Result<PostId, ApiError>>, Option<Result<PostId, ApiError>>);
///Result of Post.
pub struct PostResult {
    inner: PostResultInner,
}

impl PostResult {
    ///Retrieves Twitter's result
    pub fn twitter(&mut self) -> Option<Result<PostId, ApiError>> {
        self.inner.0.take()
    }

    ///Retrieves Gab's result
    pub fn gab(&mut self) -> Option<Result<PostId, ApiError>> {
        self.inner.1.take()
    }

    ///Retrieves Mastodon's result
    pub fn mastodon(&mut self) -> Option<Result<PostId, ApiError>> {
        self.inner.2.take()
    }

    ///Retrieves underlying errors.
    ///
    ///Order: Twitter, Gab, Mastodon
    pub fn into_parts(self) -> PostResultInner {
        self.inner
    }
}

///API access
pub struct API {
    twitter: Option<Twitter>,
    gab: Option<Gab>,
    mastodon: Option<Mastodon>,
    _http: HttpRuntime,
}

impl API {
    ///Creates new API access module by reading configuration data.
    pub fn new(settings: config::Settings) -> Self {
        Self {
            twitter: None,
            mastodon: None,
            gab: None,
            _http: http::init(&settings)
        }
    }

    ///Enables specified API by providing it configuration
    ///
    ///Does nothing if already enabled
    pub fn enable<T: ApiEnabler>(&mut self, config: T) -> Result<(), ApiError> {
        T::enable(self, config)
    }

    ///Sends Post to enabled APIs (blocking)
    pub fn send(&self, post: Post) -> Result<PostResult, ApiError> {
        let Post { message, tags, flags, mut images } = post;

        let message = if tags.len() > 0 {
            match message.as_str() {
                "" => join_hash_tags(&tags),
                message => format!("{}\n{}", message, join_hash_tags(&tags)),
            }
        } else {
            message
        };

        let message = message.as_str();
        let flags = &flags;

        let inner: Result<PostResultInner, ()> = match images {
            Some(ref mut images) if images.len() > 0 => {
                let images = {
                    let mut result = vec![];
                    for image in images.drain(..) {
                        match crate::data::Image::open(&image) {
                            Ok(image) => result.push(image),
                            Err(error) => {
                                return Err(ApiError::CannotLoadImage(image, error));
                            },
                        };
                    }
                    result
                };

                let twitter = match self.twitter {
                    Some(ref twitter) => {
                        let mut uploads = vec![];
                        for image in images.iter() {
                            let upload = twitter.upload_image(&image.name, &image.mime, &image.mmap[..]);
                            uploads.push(upload);
                        }

                        let uploads = future::join_all(uploads).map_err(|error| ApiError::Twitter(error))
                                                               .and_then(move |uploads| twitter.post(&message, &uploads, &flags).from_err())
                                                               .map(|res| Some(Ok(res)))
                                                               .or_else(|err| Ok(Some(Err(err))));
                        future::Either::A(uploads)
                    },
                    None => future::Either::B(future::ok(None))
                };

                //Twitter
                twitter.join3(
                    //Gab
                    if let Some(ref gab) = self.gab {
                        let mut uploads = vec![];
                        for image in images.iter() {
                            let upload = gab.upload_image(&image.name, &image.mime, &image.mmap[..]);
                            uploads.push(upload);
                        }

                        let uploads = future::join_all(uploads).map_err(|error| ApiError::Gab(error))
                                                               .and_then(move |uploads| gab.post(&message, &uploads, &flags).from_err())
                                                               .map(|res| Some(Ok(res)))
                                                               .or_else(|err| Ok(Some(Err(err))));
                        future::Either::A(uploads)
                    } else {
                        future::Either::B(future::ok(None))
                    },
                    //Mastodon
                    if let Some(ref mastodon) = self.mastodon {
                        let mut uploads = vec![];
                        for image in images.iter() {
                            let upload = mastodon.upload_image(&image.name, &image.mime, &image.mmap[..]);
                            uploads.push(upload);
                        }

                        let uploads = future::join_all(uploads).map_err(|error| ApiError::Mastodon(error))
                                                               .and_then(move |uploads| mastodon.post(&message, &uploads, &flags).from_err())
                                                               .map(|res| Some(Ok(res)))
                                                               .or_else(|err| Ok(Some(Err(err))));
                        future::Either::A(uploads)
                    } else {
                        future::Either::B(future::ok(None))
                    },
                ).finish()
            },
            _ => {
                //Twitter
                let twitter = match self.twitter {
                    Some(ref twitter) => {
                        let post = twitter.post(&message, &[], &flags).map_err(|error| ApiError::Twitter(error))
                                          .map(|res| Some(Ok(res)))
                                          .or_else(|err| Ok(Some(Err(err))));

                        future::Either::A(post)
                    },
                    None => future::Either::B(future::ok(None))
                };

                twitter.join3(
                    //Gab
                    if let Some(ref gab) = self.gab {
                        let post = gab.post(&message, &[], &flags).map_err(|error| ApiError::Gab(error))
                                      .map(|res| Some(Ok(res)))
                                      .or_else(|err| Ok(Some(Err(err))));

                        future::Either::A(post)

                    } else {
                        future::Either::B(future::ok(None))
                    },
                    //Mastodon
                    if let Some(ref mastodon) = self.mastodon {
                        let post = mastodon.post(&message, &[], &flags).map_err(|error| ApiError::Mastodon(error))
                                           .map(|res| Some(Ok(res)))
                                           .or_else(|err| Ok(Some(Err(err))));

                        future::Either::A(post)
                    } else {
                        future::Either::B(future::ok(None))
                    },
                ).finish()
            }
        };

        Ok(PostResult {
            inner: inner.expect("Successful post")
        })
    }
}

///Describes how to enable API
pub trait ApiEnabler {
    ///Enables API
    fn enable(api: &mut API, config: Self) -> Result<(), ApiError>;
}

impl ApiEnabler for crate::config::Mastodon {
    fn enable(api: &mut API, config: Self) -> Result<(), ApiError> {
        if api.mastodon.is_some() {
            return Ok(());
        }

        api.mastodon = Some(Mastodon::new(config)?);
        Ok(())
    }
}

impl ApiEnabler for crate::config::Gab {
    fn enable(api: &mut API, config: Self) -> Result<(), ApiError> {
        if api.gab.is_some() {
            return Ok(());
        }

        api.gab = Some(Gab::new(config)?);
        Ok(())
    }
}

impl ApiEnabler for crate::config::Twitter {
    fn enable(api: &mut API, config: Self) -> Result<(), ApiError> {
        if api.twitter.is_some() {
            return Ok(());
        }

        api.twitter = Some(Twitter::new(config)?);
        Ok(())
    }
}
