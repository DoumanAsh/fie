extern crate clap;

extern crate serde;
extern crate serde_json;
extern crate toml;
#[macro_use]
extern crate serde_derive;

extern crate egg_mode;
extern crate tokio_core;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate mime_guess;

use futures::future::Future;
use futures::Stream;
use tokio_core::reactor::Core;

#[macro_use]
mod utils;
mod config;
mod cli;
mod api;

fn init_http() -> Result<(Core, api::http::HttpClient), String> {
    let tokio_core = Core::new().map_err(error_formatter!("Unable to create tokio's event loop."))?;
    let http = api::http::create_client(&tokio_core.handle());

    Ok((tokio_core, http))
}

struct ApiConfigs {
    gab: config::Gab,
    twitter: config::Twitter,
    minds: config::Minds
}

type InitReturn<'a> = (Option<api::gab::Client<'a>>, Option<api::twitter::Client>, Option<api::minds::Client<'a>>);
fn init_api<'a>(mut tokio_core: &mut Core, http: &'a api::http::HttpClient, config: ApiConfigs, flags: cli::Flags) -> Result<InitReturn<'a>, String> {
    let gab = match flags.gab {
        true => Some(api::gab::Client::new(http, config.gab)),
        false => None
    };
    let twitter = match flags.twitter {
        true => Some(api::twitter::Client::new(tokio_core.handle(), config.twitter)),
        false => None
    };
    let minds = match flags.minds {
        true => Some(api::minds::Client::new(http, &mut tokio_core, config.minds)?),
        false => None
    };

    Ok((gab, twitter, minds))
}

fn run() -> Result<i32, String> {
    let config = config::Config::from_file(&utils::get_config())?;
    let args = cli::Args::new(config.platforms)?;
    let config = ApiConfigs {
        gab: config.gab,
        twitter: config.twitter,
        minds: config.minds,
    };

    match args.command {
        cli::Commands::Post(ref message, None) => {
            let (mut tokio_core, http) = init_http()?;
            let (gab, twitter, minds) = init_api(&mut tokio_core, &http, config, args.flags)?;
            let mut jobs: Vec<Box<Future<Item=(), Error=()>>> = vec![];

            if let Some(ref gab) = gab {
                let gab_post = gab.post(&message, &[]).map_err(error_formatter!("Cannot post.")).then(api::gab::Client::handle_post);
                jobs.push(Box::new(gab_post))
            }
            if let Some(ref twitter) = twitter {
                let tweet = twitter.post(&message, &[]).map_err(error_formatter!("Cannot tweet.")).then(api::twitter::Client::handle_post);
                jobs.push(Box::new(tweet))
            }
            if let Some(ref minds) = minds {
                let minds_post = minds.post(&message, None).map_err(error_formatter!("Cannot post.")).then(api::minds::Client::handle_post);
                jobs.push(Box::new(minds_post))
            }

            tokio_core.run(futures::future::join_all(jobs)).unwrap();
        },
        cli::Commands::Post(ref message, Some(ref images)) => {
            let images = {
                let mut result = vec![];
                for image in images {
                    result.push(utils::open_image(image).map_err(error_formatter!("Cannot open image!"))?);
                }
                result
            };

            let (mut tokio_core, http) = init_http()?;
            let (gab, twitter, minds) = init_api(&mut tokio_core, &http, config, args.flags)?;
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

                let gab_post = futures::future::join_all(gab_images).and_then(move |images| gab.post(&message, &images).map_err(error_formatter!("Cannot post.")))
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
                                   .and_then(move |images| twitter.post(&message, &images).map_err(error_formatter!("Cannot tweet.")))
                                   .then(api::twitter::Client::handle_post);
                jobs.push(Box::new(tweet))
            }
            if let Some(ref minds) = minds {
                let message = message.as_str();
                //TODO: For now Minds.com accepts only one attachment.
                if images.len() > 1 {
                    println!("Minds.com accepts only one attachment, only first image will be attached");
                }
                let minds_post = minds.upload_image(&images[0]).map_err(error_formatter!("Cannot upload image."))
                                      .and_then(handle_bad_hyper_response!("Cannot upload image."))
                                      .and_then(|response| response.body().concat2().map_err(error_formatter!("Cannot read image upload's response")))
                                      .and_then(|body| serde_json::from_slice(&body).map_err(error_formatter!("Cannot parse image upload's response")))
                                      .map(|response: api::minds::payload::UploadResponse| response.guid)
                                      .and_then(move |image| minds.post(&message, Some(image)).map_err(error_formatter!("Cannot post.")))
                                      .then(api::minds::Client::handle_post);

                jobs.push(Box::new(minds_post))
            }

            tokio_core.run(futures::future::join_all(jobs)).unwrap();
        }
    };

    Ok(0)
}

fn main() {
    use std::process::exit;

    let code: i32 = match run() {
        Ok(res) => res,
        Err(error) => {
            eprintln!("{}", error);
            1
        }
    };

    exit(code);
}

