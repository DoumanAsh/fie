#[macro_use]
extern crate serde_derive;
extern crate actix;

use actix::{Actor};

mod misc;
mod io;
mod config;
mod actors;
mod cli;

fn run() -> Result<i32, String> {
    let config = config::Config::from_default_config()?;
    let args = cli::Args::new(config.platforms)?;

    match args.command {
        cli::Commands::Post(post) => {
            let mut system = actix::System::new("fie");
            let api: actix::Addr<actix::Unsync, _> = actors::API::new().start_minds_if(args.flags.minds, config.minds)
                                                                       .start_gab_if(args.flags.gab, config.gab)
                                                                       .start_twitter_if(args.flags.twitter, config.twitter)
                                                                       .start();

            let _ = system.run_until_complete(api.send(post));
            Ok(0)
        },
        cli::Commands::Batch(exec) => match exec.post {
            Some(posts) => {
                let mut system = actix::System::new("fie");
                let api: actix::Addr<actix::Unsync, _> = actors::API::new().start_minds_if(args.flags.minds, config.minds)
                                                                           .start_gab_if(args.flags.gab, config.gab)
                                                                           .start_twitter_if(args.flags.twitter, config.twitter)
                                                                           .start();
                for (idx, post) in posts.into_iter().enumerate() {
                    println!(">>>Post #{}", idx + 1);
                    let _ = system.run_until_complete(api.send(post));
                }
                Ok(0)
            }
            None => Ok(0),
        },
        cli::Commands::Env(env) => match env {
            cli::EnvCommand::Config => {
                println!("{}", config::Config::default_config_path().display());
                Ok(0)
            }
        }
    }
}

fn main() {
    use std::process::exit;

    let ret = match run() {
        Ok(ret) => ret,
        Err(error) => {
            eprintln!("{}", error);
            1
        }
    };

    exit(ret)
}

