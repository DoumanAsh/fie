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

use std::convert;

#[macro_use]
mod utils;
mod config;
mod cli;
mod api;
mod exec;

fn run() -> Result<i32, String> {
    let config_path = utils::get_config();
    let config = config::Config::from_file(&config_path)?;
    let args = cli::Args::new(config.platforms)?;
    let config = exec::ApiConfigs {
        gab: config.gab,
        twitter: config.twitter,
        minds: config.minds,
        platforms: args.flags
    };

    match args.command {
        cli::Commands::Post(post) => exec::post(post, config),
        cli::Commands::Batch(exec) => exec::batch(exec, config),
        cli::Commands::Env(env) => exec::env(env, config_path)
    }?;

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

