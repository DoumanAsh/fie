extern crate egg_mode;
extern crate clap;

use std::env;

mod api;
mod cli;

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

    let token = api::twitter::create_token(access_key, access_secret);

    match args.command {
        cli::Commands::Post(message, tags) => {
            match api::twitter::post_tweet(message, tags, &token) {
                Ok(tweet) => println!("Posted tweet:\n{}", tweet.text),
                Err(error) => return Err(format!("Failed to post tweet. Error: {}", error))
            }
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

