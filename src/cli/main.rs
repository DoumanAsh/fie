#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

use serde_derive::Deserialize;

mod config;
mod cli;
mod auth;

use fie::config::Config;
use config::FileSystemLoad;

use std::io;
use std::path::Path;

fn create_api(config: Config) -> io::Result<fie::API> {
    let mut any_enabled = false;
    let mut api = fie::API::new(config.settings);

    if config.platforms.gab {
        if let Err(error) = api.configure(config.api.gab) {
            eprintln!("{}", error);
        } else {
            any_enabled = true
        }
    }

    if config.platforms.twitter {
        if let Err(error) = api.configure(config.api.twitter) {
            eprintln!("{}", error);
        } else {
            any_enabled = true
        }
    }

    if config.platforms.mastodon {
        if let Err(error) = api.configure(config.api.mastodon) {
            eprintln!("{}", error);
        } else {
            any_enabled = true
        }
    }

    if config.platforms.minds {
        if let Err(error) = api.configure(config.api.minds) {
            eprintln!("{}", error);
        } else {
            any_enabled = true
        }
    }

    match any_enabled {
        true => Ok(api),
        false => Err(io::Error::new(io::ErrorKind::Other, "No API is enabled :(")),
    }
}

fn handle_post_result(result: fie::api::PostResult) {
    let (twitter, gab, mastodon, minds) = result.into_parts();

    let handle_inner = |prefix, result| if let Some(result) = result {
        match result {
            Ok(id) => println!("{}(Id={})", prefix, id),
            Err(error) => eprintln!("{}", error)
        }
    };

    handle_inner("Twitter", twitter);
    handle_inner("Gab", gab);
    handle_inner("Mastodon", mastodon);
    handle_inner("Minds", minds);
}

#[derive(Deserialize, Debug)]
pub struct Batch {
    post: Vec<fie::data::Post>,
}

fn open_batch(path: &str) -> io::Result<Batch> {
    config::load_from_file(Path::new(path))
}

fn use_twitter_builtin_consumer(twitter: &mut fie::config::Twitter) {
    const CONSUMER_KEY: Option<&'static str> = option_env!("FIE_TWITTER_CONSUMER_KEY");
    const CONSUMER_SECRET: Option<&'static str> = option_env!("FIE_TWITTER_CONSUMER_SECRET");

    //Only set if either part of consumer token is missing
    match (CONSUMER_KEY, CONSUMER_SECRET) {
        (Some(key), Some(secret)) => if twitter.consumer.key.len() == 0 || twitter.consumer.secret.len() == 0{
            twitter.consumer.key.truncate(0);
            twitter.consumer.secret.truncate(0);

            twitter.consumer.key.push_str(key);
            twitter.consumer.secret.push_str(secret);
        },
        _ => (),
    }
}

fn run() -> io::Result<()> {
    let mut config = Config::load()?;
    use_twitter_builtin_consumer(&mut config.api.twitter);

    let args = cli::Args::new(&mut config.platforms);

    match args.cmd {
        cli::Command::Post(post) => {
            let result = create_api(config)?.send(post.into()).map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
            handle_post_result(result);
        },
        cli::Command::Batch(batch) => {
            let api = create_api(config)?;

            for (idx, post) in open_batch(&batch.file)?.post.drain(..).enumerate() {
                println!(">>>Post #{}:", idx + 1);
                match api.send(post) {
                    Ok(result) => handle_post_result(result),
                    Err(error) => eprintln!("{}", error),
                }
            }
        },
        cli::Command::Env(env) => match env {
            cli::Env::Config => println!("{}", Config::path()?.display())
        },
        cli::Command::Auth(typ) => match typ {
            cli::Auth::Twitter => {
                auth::twitter(config.api.twitter);
            }
        }
    }

    Ok(())
}

fn main() {
    let result = match run() {
        Ok(_) => 0,
        Err(error) => {
            eprintln!("{}", error);
            1
        }
    };

    std::process::exit(result);
}
