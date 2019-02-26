#[macro_use]
extern crate serde_derive;

mod api;
mod cli;
mod config;
mod data;
mod http;
mod io;
mod misc;

fn run() -> Result<i32, String> {
    let config = config::Config::from_default_config()?;
    let args = cli::Args::new(config.platforms)?;

    match args.command {
        cli::Commands::Post(post) => {
            let api = api::API::new(config.settings)
                .start_gab_if(args.flags.gab, config.gab)
                .start_minds_if(args.flags.minds, config.minds)
                .start_twitter_if(args.flags.twitter, config.twitter)
                .start_mastodon_if(args.flags.mastodon, config.mastodon);

            api.send(post);
        },
        cli::Commands::Batch(exec) => match exec.post {
            Some(posts) => {
                let api = api::API::new(config.settings)
                    .start_gab_if(args.flags.gab, config.gab)
                    .start_minds_if(args.flags.minds, config.minds)
                    .start_twitter_if(args.flags.twitter, config.twitter)
                    .start_mastodon_if(args.flags.mastodon, config.mastodon);

                for (idx, post) in posts.into_iter().enumerate() {
                    println!(">>>Post #{}", idx + 1);
                    api.send(post);
                }
            },
            None => (),
        },
        cli::Commands::Env(env) => match env {
            cli::EnvCommand::Config => println!("{}", config::Config::default_config_path().display()),
        },
    }

    Ok(0)
}

fn main() {
    use std::process::exit;

    let ret = match run() {
        Ok(ret) => ret,
        Err(error) => {
            eprintln!("{}", error);
            1
        },
    };

    exit(ret)
}
