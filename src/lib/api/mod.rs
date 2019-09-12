//!Social medias API module

pub mod http;
pub mod twitter;
pub mod gab;
pub mod mastodon;
pub mod minds;

use twitter::{Twitter, TwitterError};
use gab::{Gab, GabError};
use mastodon::{Mastodon, MastodonError};
use minds::{Minds, MindsError};
use http::{matsu};
use crate::data::{join_hash_tags, PostId, Post};

use super::config;

use core::fmt;
use std::error::Error;
use std::io;

#[derive(Debug)]
///API Errors
pub enum ApiError {
    ///Unable to load Image for attachment
    CannotLoadImage(String, io::Error),
    ///Twitter error
    Twitter(TwitterError),
    ///Gab error
    Gab(GabError),
    ///Mastodon error
    Mastodon(MastodonError),
    ///Minds error
    Minds(MindsError),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ApiError::CannotLoadImage(ref name, ref error) => write!(f, "Error opening image '{}'. Error: {}", name, error),
            &ApiError::Twitter(ref error) => write!(f, "Twitter API Error: {}", error),
            &ApiError::Gab(ref error) => write!(f, "Gab API Error: {}", error),
            &ApiError::Mastodon(ref error) => write!(f, "Mastodon API Error: {}", error),
            &ApiError::Minds(ref error) => write!(f, "MindsError API Error: {}", error),
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

impl From<MindsError> for ApiError {
    fn from(error: MindsError) -> Self {
        ApiError::Minds(error)
    }
}

type PostResultInner = (Option<Result<PostId, ApiError>>, Option<Result<PostId, ApiError>>, Option<Result<PostId, ApiError>>, Option<Result<PostId, ApiError>>);

async fn post_result<T, E: Into<ApiError>, F: core::future::Future<Output=Result<T, E>>>(post: Option<F>) -> Option<Result<T, ApiError>> {
    match post {
        Some(post) => Some(matsu!(post).map_err(|err| err.into())),
        None => None,
    }
}

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

    ///Retrieves Minds's result
    pub fn minds(&mut self) -> Option<Result<PostId, ApiError>> {
        self.inner.3.take()
    }

    ///Retrieves underlying errors.
    ///
    ///Order: Twitter, Gab, Mastodon, Minds
    pub fn into_parts(self) -> PostResultInner {
        self.inner
    }
}

///API access
pub struct API {
    twitter: Option<Twitter>,
    gab: Option<Gab>,
    mastodon: Option<Mastodon>,
    minds: Option<Minds>,
}

impl API {
    ///Creates new API access module by reading configuration data.
    pub fn new(settings: config::Settings) -> Self {
        http::set_timeout(&settings);
        Self {
            twitter: None,
            mastodon: None,
            gab: None,
            minds: None,
        }
    }

    ///Performs initial configuration of Twitter API.
    pub fn configure_twitter(&mut self, config: config::Twitter) -> Result<(), ApiError> {
        if self.twitter.is_some() {
            return Ok(());
        }

        self.twitter = Some(Twitter::new(config)?);
        Ok(())
    }

    ///Enables twitter back, if it was enabled
    pub fn enable_twitter(&mut self, twitter: Option<Twitter>) {
        self.twitter = twitter;
    }

    ///Disables twitter.
    pub fn disable_twitter(&mut self) -> Option<Twitter> {
        self.twitter.take()
    }

    ///Performs initial configuration of Gab API.
    pub fn configure_gab(&mut self, config: config::Gab) -> Result<(), ApiError> {
        if self.gab.is_some() {
            return Ok(());
        }

        self.gab = Some(Gab::new(config)?);
        Ok(())
    }

    ///Enables Gab back, if it was enabled
    pub fn enable_gab(&mut self, gab: Option<Gab>) {
        self.gab = gab;
    }

    ///Disables Gab.
    pub fn disable_gab(&mut self) -> Option<Gab> {
        self.gab.take()
    }

    ///Performs initial configuration of Gab API.
    pub fn configure_mastodon(&mut self, config: config::Mastodon) -> Result<(), ApiError> {
        if self.mastodon.is_some() {
            return Ok(());
        }

        self.mastodon = Some(Mastodon::new(config)?);
        Ok(())
    }

    ///Enables Mastodon back, if it was enabled
    pub fn enable_mastodon(&mut self, mastodon: Option<Mastodon>) {
        self.mastodon = mastodon;
    }

    ///Disables Mastodon.
    pub fn disable_mastodon(&mut self) -> Option<Mastodon> {
        self.mastodon.take()
    }

    ///Performs initial configuration of Minds API.
    pub async fn configure_minds(&mut self, config: config::Minds) -> Result<(), ApiError> {
        if self.minds.is_some() {
            return Ok(());
        }

        self.minds = Some(matsu!(Minds::new(config))?);
        Ok(())
    }

    ///Enables Minds back, if it was enabled
    pub fn enable_minds(&mut self, minds: Option<Minds>) {
        self.minds = minds;
    }

    ///Disables Minds.
    pub fn disable_minds(&mut self) -> Option<Minds> {
        self.minds.take()
    }

    ///Sends Post to enabled APIs (blocking)
    pub async fn send(&self, post: Post) -> Result<PostResult, ApiError> {
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

        let inner = match images.len() {
            0 => {
                let twitter = post_result(self.twitter.as_ref().map(|twitter| twitter.post(&message, &[], &flags)));
                let gab = post_result(self.gab.as_ref().map(|gab| gab.post(&message, &[], &flags)));
                let mastodon = post_result(self.mastodon.as_ref().map(|mastodon| mastodon.post(&message, &[], &flags)));
                let minds = post_result(self.minds.as_ref().map(|minds| minds.post(&message, None, &flags)));
                futures_util::join!(twitter, gab, mastodon, minds)
            },
            _ => {
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

                let images = &images[..];

                let twitter = post_result(self.twitter.as_ref().map(async move |twitter| {
                    let mut uploads = vec![];
                    for image in images.iter() {
                        let upload = matsu!(twitter.upload_image(&image.name, &image.mime, &image.mmap[..]))?;
                        uploads.push(upload);
                    }

                    matsu!(twitter.post(&message, &uploads, &flags))
                }));

                let gab = post_result(self.gab.as_ref().map(async move |gab| {
                    let mut uploads = vec![];
                    for image in images.iter() {
                        let upload = matsu!(gab.upload_image(&image.name, &image.mime, &image.mmap[..]))?;
                        uploads.push(upload);
                    }

                    matsu!(gab.post(&message, &uploads, &flags))
                }));

                let mastodon = post_result(self.mastodon.as_ref().map(async move |mastodon| {
                    let mut uploads = vec![];
                    for image in images.iter() {
                        let upload = matsu!(mastodon.upload_image(&image.name, &image.mime, &image.mmap[..]))?;
                        uploads.push(upload);
                    }

                    matsu!(mastodon.post(&message, &uploads, &flags))
                }));

                let minds = post_result(self.minds.as_ref().map(async move |minds| {
                    let image = unsafe { images.get_unchecked(0) };
                    let upload = matsu!(minds.upload_image(&image.name, &image.mime, &image.mmap[..]))?;

                    matsu!(minds.post(&message, Some(upload), &flags))
                }));

                let res = futures_util::join!(twitter, gab, mastodon, minds);
                res
            }
        };

        Ok(PostResult {
            inner,
        })
    }
}
