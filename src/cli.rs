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
                                                .multiple(true)
                                                .help("Adds hashtag at the last line of post."))
                                 .arg(arg("image").short("i")
                                                  .multiple(true)
                                                  .takes_value(true)
                                                  .help("Adds image to post. Normally up to 4."))
}

pub fn parser() -> App<'static, 'static> {
    App::new(NAME).about(ABOUT)
                  .author(AUTHOR)
                  .version(VERSION)
                  .setting(AppSettings::ArgRequiredElseHelp)
                  .setting(AppSettings::VersionlessSubcommands)
                  .subcommand(new_command())
                  .arg(flag("gab").help("Use gab.ai. By default all social medias are used unless flag is specified."))
                  .arg(flag("twitter").help("Use Twitter. By default all social medias are used unless flag is specified."))
                  .arg(flag("minds").help("Use Minds.com. By default all social medias are used unless flag is specified."))

}

#[derive(Debug)]
///Command representation with all its arguments.
pub enum Commands {
    ///Creates new tweet.
    ///
    ///# Parameters:
    ///
    ///* First - Text.
    ///* Second - Tags.
    ///* Third - Image to attach.
    Post(String, Option<Vec<String>>, Option<Vec<String>>)
}

impl Commands {
    fn from_matches(sub_command: (&str, Option<&ArgMatches<'static>>)) -> Self {
        let (name, matches) = sub_command;
        let matches = matches.unwrap();

        match name {
            "post" => {
                let message = matches.value_of("message").unwrap().to_string();
                let tags = matches.values_of("tag").map(|values| values.map(|tag| format!("#{}", tag)).collect());
                let image = matches.values_of("image").map(|images| images.map(|image| image.to_string()).collect());

                Commands::Post(message, tags, image)
            },
            _ => unimplemented!()
        }
    }
}

#[derive(Debug)]
pub struct Flags {
    ///Whether to use gab.ai
    pub gab: bool,
    ///Whether to use Twitter
    pub twitter: bool,
    ///Whether to use Minds
    pub minds: bool,
}

impl Flags {
    fn from_matches(matches: &ArgMatches<'static>) -> Self {
        let mut gab = matches.is_present("gab");
        let mut twitter = matches.is_present("twitter");
        let mut minds = matches.is_present("minds");

        if !gab && !twitter && !minds {
            gab = true;
            twitter = true;
            minds = true
        }

        Flags {
            gab,
            twitter,
            minds
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
    pub fn new() -> Result<Self, String> {
        let matches = parser().get_matches();
        let command = Commands::from_matches(matches.subcommand());
        let flags = Flags::from_matches(&matches);

        Ok(Args {
            command,
            flags
        })
    }
}
