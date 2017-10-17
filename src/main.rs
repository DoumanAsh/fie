extern crate egg_mode;
extern crate tokio_core;
extern crate futures;
extern crate clap;

use tokio_core::reactor::Core;

use std::env;

mod api;
mod cli;
#[macro_use]
mod utils;

fn run() -> Result<i32, String> {
    let args = cli::Args::new()?;

    let access_token = match env::var(api::twitter::ACCESS_TOKEN_ENV) {
        Ok(result) => result,
        Err(error) => return Err(format!("Twitter access token '{}' is not available. Error: {}", api::twitter::ACCESS_TOKEN_ENV, error)),
    };
    let mut access_split = access_token.split_whitespace();
    let access_key = match access_split.next() {
        Some(result) => result,
        None => return Err("Access token key is missing. Access token format: <key> <secrect>".to_string())
    };
    let access_secret = match access_split.next() {
        Some(result) => result,
        None => return Err("Access token secret is missing. Access token format: <key> <secrect>".to_string())
    };

    let mut tokio_core = Core::new().map_err(error_formatter!("Unable to create tokios' event loop."))?;
    let twitter = api::twitter::Client::new(tokio_core.handle(), access_key, access_secret);

    match args.command {
        cli::Commands::Post(message, tags) => {
            let rsp = tokio_core.run(twitter.post(message, tags)).map_err(error_formatter!("Cannot post error"))?;
            println!("Posted tweet(id={}):\n{}\n", rsp.response.id, rsp.response.text);
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

