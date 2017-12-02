use ::clap::{App, Arg, SubCommand, AppSettings, ArgMatches};

use ::std::fmt::Display;
use ::std::str::FromStr;

use ::config::Platforms;

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
fn post_command() -> App<'static, 'static> {
    SubCommand::with_name("post").about("Creates new tweet.")
                                 .arg(arg("message").required(true)
                                                    .help("Message content"))
                                 .arg(arg("tag").short("t")
                                                .takes_value(true)
                                                .number_of_values(1)
                                                .multiple(true)
                                                .help("Adds hashtag at the last line of post."))
                                 .arg(arg("image").short("i")
                                                  .multiple(true)
                                                  .takes_value(true)
                                                  .help("Adds image to post. Normally up to 4."))
                                 .arg(flag("nsfw").help("Whether post is NSFW or not."))
}

#[inline(always)]
fn env_config_command() -> App<'static, 'static> {
    SubCommand::with_name("config").about("Prints path to config file.")
}

#[inline(always)]
fn env_command() -> App<'static, 'static> {
    SubCommand::with_name("env").about("Prints information about app environment.")
                                .setting(AppSettings::ArgRequiredElseHelp)
                                .subcommand(env_config_command())
}

pub fn parser() -> App<'static, 'static> {
    App::new(NAME).about(ABOUT)
                  .author(AUTHOR)
                  .version(VERSION)
                  .setting(AppSettings::ArgRequiredElseHelp)
                  .setting(AppSettings::VersionlessSubcommands)
                  .arg(flag("gab").help("Use gab.ai. By default all social medias are used unless flag is specified."))
                  .arg(flag("twitter").help("Use Twitter. By default all social medias are used unless flag is specified."))
                  .arg(flag("minds").help("Use Minds.com. By default all social medias are used unless flag is specified."))
                  .subcommand(post_command())
                  .subcommand(env_command())

}

#[derive(Debug)]
///Env subcommand variants
pub enum EnvCommand {
    ///Prints configuration file.
    Config
}

#[derive(Debug)]
pub struct PostFlags {
    pub nsfw: bool
}

#[derive(Debug)]
///Command representation with all its arguments.
pub enum Commands {
    ///Creates new tweet.
    ///
    ///# Parameters:
    ///
    ///* First - Text.
    ///* Second - Images to attach.
    Post(String, PostFlags, Option<Vec<String>>),
    ///Prints environment information.
    Env(EnvCommand)

}

impl Commands {
    fn from_matches(sub_command: (&str, Option<&ArgMatches<'static>>)) -> Self {
        let (name, matches) = sub_command;
        let matches = matches.unwrap();

        match name {
            "post" => {
                let message = matches.value_of("message").unwrap();
                let image = matches.values_of("image").map(|images| images.map(|image| image.to_string()).collect());
                let message = match matches.values_of("tag") {
                    Some(tags) => format!("{}\n{}", message, tags.map(|value| format!("#{}", value)).collect::<Vec<String>>().join(" ")),
                    None => message.to_string()
                };
                let flags = PostFlags {
                    nsfw: matches.is_present("nsfw")
                };

                Commands::Post(message, flags, image)
            },
            "env" => {
                match matches.subcommand() {
                    ("config", _) => Commands::Env(EnvCommand::Config),
                    _ => unimplemented!()
                }
            }
            _ => unimplemented!()
        }
    }
}

pub type Flags = Platforms;

impl Flags {
    fn from_matches(matches: &ArgMatches<'static>) -> Option<Self> {
        let gab = matches.is_present("gab");
        let twitter = matches.is_present("twitter");
        let minds = matches.is_present("minds");

        if !gab && !twitter && !minds {
            None
        }
        else {
            Some(Flags {
                gab,
                twitter,
                minds
            })
        }
    }
}

#[derive(Debug)]
pub struct Args {
    ///Command to execute
    pub command: Commands,
    pub flags: Flags
}

impl Args {
    pub fn new(platforms: Platforms) -> Result<Self, String> {
        let matches = parser().get_matches();
        let command = Commands::from_matches(matches.subcommand());
        let flags = Flags::from_matches(&matches).unwrap_or(platforms);

        Ok(Args {
            command,
            flags
        })
    }
}
