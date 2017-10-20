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

use futures::future::Future;
use tokio_core::reactor::Core;

use std::env;
use std::path;

mod api;
mod cli;
#[macro_use]
mod utils;
mod config;

fn get_config() -> path::PathBuf {
    let mut result = env::current_exe().unwrap();

    result.set_file_name(config::NAME);

    result
}

fn run() -> Result<i32, String> {
    let config = config::Config::from_file(&get_config())?;
    let args = cli::Args::new()?;

    let mut tokio_core = Core::new().map_err(error_formatter!("Unable to create tokios' event loop."))?;
    let twitter = api::twitter::Client::new(tokio_core.handle(), config.twitter);
    let gab = api::gab::Client::new(tokio_core.handle(), config.gab);

    match args.command {
        cli::Commands::Post(message, tags) => {
            tokio_core.run(gab.post(&message, &tags).map_err(error_formatter!("Cannot post.")).and_then(api::gab::Client::handle_post))?;
            tokio_core.run(twitter.post(&message, &tags).map_err(error_formatter!("Cannot tweet.")).and_then(api::twitter::Client::handle_post))?
        },
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

