use std::path::PathBuf;

use ::tokio_core::reactor::Core;
use ::futures::future::Future;
use ::futures::Stream;
use ::futures;
use ::hyper;
use ::serde_json;

use ::utils;
use ::cli;
use ::config;
use ::api;

pub fn post(message: String, flags: cli::PostFlags, images: Option<Vec<String>>, config: ApiConfigs) -> Result<(), String> {
    match images {
        Some(images) => post_w_image(&message, &flags, &images, config),
        None => post_no_image(&message, &flags, config),
    }
}

pub fn env(env: cli::EnvCommand, config_path: PathBuf) -> Result<(), String> {
    match env {
        cli::EnvCommand::Config => println!("{}", config_path.display()),
    }

    Ok(())
}

fn init_http() -> Result<(Core, api::http::HttpClient), String> {
    let tokio_core = Core::new().map_err(error_formatter!("Unable to create tokio's event loop."))?;
    let http = api::http::create_client(&tokio_core.handle());

    Ok((tokio_core, http))
}

pub struct ApiConfigs {
    pub gab: config::Gab,
    pub twitter: config::Twitter,
    pub minds: config::Minds,
    pub platforms: config::Platforms
}

type InitReturn<'a> = (Option<api::gab::Client<'a>>, Option<api::twitter::Client>, Option<api::minds::Client<'a>>);
fn init_api<'a>(mut tokio_core: &mut Core, http: &'a api::http::HttpClient, config: ApiConfigs) -> Result<InitReturn<'a>, String> {
    let gab = match config.platforms.gab {
        true => Some(api::gab::Client::new(http, config.gab)),
        false => None
    };
    let twitter = match config.platforms.twitter {
        true => Some(api::twitter::Client::new(tokio_core.handle(), config.twitter)),
        false => None
    };
    let minds = match config.platforms.minds {
        true => Some(api::minds::Client::new(http, &mut tokio_core, config.minds)?),
        false => None
    };

    Ok((gab, twitter, minds))
}

#[inline(always)]
pub fn post_no_image(message: &str, flags: &cli::PostFlags, config: ApiConfigs) -> Result<(), String> {
    let (mut tokio_core, http) = init_http()?;
    let (gab, twitter, minds) = init_api(&mut tokio_core, &http, config)?;
    let mut jobs: Vec<Box<Future<Item=(), Error=()>>> = vec![];

    if let Some(ref gab) = gab {
        let gab_post = gab.post(&message, &flags, &[]).map_err(error_formatter!("Cannot post.")).then(api::gab::Client::handle_post);
        jobs.push(Box::new(gab_post))
    }
    if let Some(ref twitter) = twitter {
        let tweet = twitter.post(&message, &flags, &[]).map_err(error_formatter!("Cannot tweet.")).then(api::twitter::Client::handle_post);
        jobs.push(Box::new(tweet))
    }
    if let Some(ref minds) = minds {
        let minds_post = minds.post(&message, &flags, None).map_err(error_formatter!("Cannot post.")).then(api::minds::Client::handle_post);
        jobs.push(Box::new(minds_post))
    }

    tokio_core.run(futures::future::join_all(jobs)).unwrap();
    Ok(())
}

#[inline(always)]
pub fn post_w_image(message: &str, flags: &cli::PostFlags, images: &[String], config: ApiConfigs) -> Result<(), String> {
    let images = {
        let mut result = vec![];
        for image in images {
            result.push(utils::open_image(image).map_err(error_formatter!("Cannot open image!"))?);
        }
        result
    };

    let (mut tokio_core, http) = init_http()?;
    let (gab, twitter, minds) = init_api(&mut tokio_core, &http, config)?;
    let mut jobs: Vec<Box<Future<Item=(), Error=()>>> = vec![];

    if let Some(gab) = gab.as_ref() {
        let mut gab_images: Vec<_> = vec![];
        for image in &images {
            gab_images.push(gab.upload_image(&image).map_err(error_formatter!("Cannot upload image."))
                            .and_then(handle_bad_hyper_response!("Cannot upload image."))
                            .and_then(|response| response.body().concat2().map_err(error_formatter!("Cannot read image upload's response")))
                            .and_then(|body| serde_json::from_slice(&body).map_err(error_formatter!("Cannot parse image upload's response")))
                            .map(|response: api::gab::payload::UploadResponse| response.id));
        }

        let gab_post = futures::future::join_all(gab_images).and_then(move |images| gab.post(&message, &flags, &images).map_err(error_formatter!("Cannot post.")))
            .then(api::gab::Client::handle_post);
        jobs.push(Box::new(gab_post))
    }
    if let Some(twitter) = twitter.as_ref() {
        let mut tweet_images: Vec<_> = vec![];
        for image in &images {
            tweet_images.push(twitter.upload_image(&image).map_err(error_formatter!("Cannot upload image."))
                              .map(|rsp| rsp.media_id));
        }

        let tweet = futures::future::join_all(tweet_images)
            .and_then(move |images| twitter.post(&message, &flags, &images).map_err(error_formatter!("Cannot tweet.")))
            .then(api::twitter::Client::handle_post);
        jobs.push(Box::new(tweet))
    }
    if let Some(ref minds) = minds {
        //TODO: For now Minds.com accepts only one attachment.
        if images.len() > 1 {
            println!("Minds.com accepts only one attachment, only first image will be attached");
        }
        let minds_post = minds.upload_image(&images[0]).map_err(error_formatter!("Cannot upload image."))
            .and_then(handle_bad_hyper_response!("Cannot upload image."))
            .and_then(|response| response.body().concat2().map_err(error_formatter!("Cannot read image upload's response")))
            .and_then(|body| serde_json::from_slice(&body).map_err(error_formatter!("Cannot parse image upload's response")))
            .map(|response: api::minds::payload::UploadResponse| response.guid)
            .and_then(move |image| minds.post(&message, &flags, Some(image)).map_err(error_formatter!("Cannot post.")))
            .then(api::minds::Client::handle_post);

        jobs.push(Box::new(minds_post))
    }

    tokio_core.run(futures::future::join_all(jobs)).unwrap();
    Ok(())
}