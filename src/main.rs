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

mod api;
mod cli;
#[macro_use]
mod utils;
mod config;

fn run() -> Result<i32, String> {
    let config = config::Config::from_file(&utils::get_config())?;
    let args = cli::Args::new()?;

    let mut tokio_core = Core::new().map_err(error_formatter!("Unable to create tokios' event loop."))?;
    let twitter = api::twitter::Client::new(tokio_core.handle(), config.twitter);
    let gab = api::gab::Client::new(tokio_core.handle(), config.gab);

    match args.command {
        cli::Commands::Post(message, tags, None) => {
            println!(">>>Gab:");
            tokio_core.run(gab.post(&message, &tags).map_err(error_formatter!("Cannot post.")).and_then(api::gab::Client::handle_post))?;
            println!(">>>Twitter:");
            tokio_core.run(twitter.post(&message, &tags).map_err(error_formatter!("Cannot tweet.")).and_then(api::twitter::Client::handle_post))?;
        },
        cli::Commands::Post(message, tags, Some(image)) => {
            let image = utils::open_image(image).map_err(error_formatter!("Cannot open image!"))?;
            println!(">>>Gab:");
            let gab_post = gab.upload_image(&image).map_err(error_formatter!("Cannot upload image."))
                              .and_then(handle_bad_hyper_response!("Cannot upload image."))
                              .and_then(|response| response.body().concat2().map_err(error_formatter!("Cannot read image upload's response")))
                              .and_then(move |body| serde_json::from_slice(&body).map_err(error_formatter!("Cannot parse image upload's response")))
                              .and_then(|response: api::gab::payload::UploadResponse| gab.post_w_images(&message, &tags, &[response.id]).map_err(error_formatter!("Cannot post.")))
                              .and_then(api::gab::Client::handle_post);
            tokio_core.run(gab_post)?;
            println!(">>>Twitter:");
            let tweet = twitter.upload_image(&image).map_err(error_formatter!("Cannot upload image."))
                               .and_then(|rsp| twitter.post_w_images(&message, &tags, &[rsp.response.id]).map_err(error_formatter!("Cannot tweet.")))
                               .and_then(api::twitter::Client::handle_post);
            tokio_core.run(tweet)?;
        }
    }

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

