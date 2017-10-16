use ::clap::{App, Arg, SubCommand, AppSettings, ArgMatches};

use ::std::fmt::Display;
use ::std::str::FromStr;

#[inline(always)]
///Shortcut to create CLI argument
fn arg(name: &str) -> Arg {
    Arg::with_name(name)
}

#[inline(always)]
///Shortcut to create CLI option/flag
fn flag(name: &str) -> Arg {
    arg(name).long(name)
}

#[inline(always)]
///Shortcut to parse integer
fn parse_int<T: FromStr>(name: &str) -> Result<T, String> where <T as FromStr>::Err: Display {
    name.parse::<T>().map_err(|error| format!("Invalid number '{}' is supplied. {}", name, error))
}

const NAME: &'static str = env!("CARGO_PKG_NAME");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const ABOUT: &'static str = "
Small and cute twitter app.";

#[inline(always)]
fn new_command() -> App<'static, 'static> {
    SubCommand::with_name("post").about("Creates new tweet")
                                 .arg(arg("message").required(true)
                                                    .help("Message content"))
                                 .arg(arg("tag").short("t")
                                                .takes_value(true)
                                                .number_of_values(1)
                                                .multiple(true))
}

pub fn parser() -> App<'static, 'static> {
    App::new(NAME).about(ABOUT)
                  .author(AUTHOR)
                  .version(VERSION)
                  .setting(AppSettings::ArgRequiredElseHelp)
                  .setting(AppSettings::VersionlessSubcommands)
                  .subcommand(new_command())

}

#[derive(Debug)]
///Command representation with all its arguments
pub enum Commands {
    ///Creates new tweet
    Post(String, Option<Vec<String>>)
}

impl Commands {
    fn from_matches(sub_command: (&str, Option<&ArgMatches<'static>>)) -> Self {
        let (name, matches) = sub_command;
        let matches = matches.unwrap();

        match name {
            "post" => {
                let message = matches.value_of("message").unwrap().to_string();
                if let Some(tags) = matches.values_of("tag") {
                    Commands::Post(message, Some(tags.map(|tag| format!("#{}", tag)).collect()))
                }
                else {
                    Commands::Post(message, None)
                }
            },
            _ => unimplemented!()
        }
    }
}

#[derive(Debug)]
pub struct Args {
    ///Command to execute
    pub command: Commands
}

impl Args {
    pub fn new() -> Result<Self, String> {
        let matches = parser().get_matches();
        let command = Commands::from_matches(matches.subcommand());

        Ok(Args {
            command
        })
    }
}
