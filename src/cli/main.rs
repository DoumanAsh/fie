#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

mod config;
mod cli;

use fie::config::Config;
use config::FileSystemLoad;

use std::io;
use std::path::Path;

fn create_api(config: Config) -> io::Result<fie::API> {
    let mut any_enabled = false;
    let mut api = fie::API::new(config.settings);

    if config.platforms.gab {
        if let Err(error) = api.enable(config.api.gab) {
            eprintln!("{}", error);
        } else {
            any_enabled = true
        }
    }

    if config.platforms.twitter {
        if let Err(error) = api.enable(config.api.twitter) {
            eprintln!("{}", error);
        } else {
            any_enabled = true
        }
    }

    if config.platforms.mastodon {
        if let Err(error) = api.enable(config.api.mastodon) {
            eprintln!("{}", error);
        } else {
            any_enabled = true
        }
    }

    if config.platforms.minds {
        if let Err(error) = api.enable(config.api.minds) {
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

fn open_batch(path: &str) -> io::Result<Vec<fie::data::Post>> {
    config::load_from_file(Path::new(path))
}

fn run() -> io::Result<()> {
    let mut config = Config::load()?;
    let args = cli::Args::new(&mut config.platforms);

    match args.cmd {
        cli::Command::Post(post) => {
            let result = create_api(config)?.send(post.into()).map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
            handle_post_result(result);
        },
        cli::Command::Batch(batch) => {
            let api = create_api(config)?;

            for (idx, post) in open_batch(&batch.file)?.drain(..).enumerate() {
                println!(">>>Post #{}:", idx + 1);
                match api.send(post) {
                    Ok(result) => handle_post_result(result),
                    Err(error) => eprintln!("{}", error),
                }
            }
        },
        cli::Command::Env(env) => match env {
            cli::Env::Config => println!("{}", Config::path()?.display())
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
