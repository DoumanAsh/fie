use std::convert;

use crate::config::Platforms;
use crate::data::PostFlags;
use crate::io;
use crate::misc::ResultExt;

mod clap;
use self::clap::{parser, ArgMatches};

#[derive(Deserialize, Debug)]
/// Env subcommand variants
pub enum EnvCommand {
    /// Prints configuration file.
    Config,
}

#[derive(Deserialize, Debug)]
pub struct Post {
    pub message: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub flags: PostFlags,
    pub images: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct Batch {
    pub post: Option<Vec<Post>>,
}

#[derive(Debug)]
/// Command representation with all its arguments.
pub enum Commands {
    /// Creates new tweet.
    ///
    /// # Parameters:
    ///
    /// * First - Text.
    /// * Second - Flags.
    /// * Third - Images to attach.
    Post(Post),
    /// Prints environment information.
    Env(EnvCommand),
    /// Executes batch of commands.
    Batch(Batch),
}

impl convert::From<EnvCommand> for Commands {
    fn from(env: EnvCommand) -> Self {
        Commands::Env(env)
    }
}

impl convert::From<Batch> for Commands {
    fn from(batch: Batch) -> Self {
        Commands::Batch(batch)
    }
}

impl convert::From<Post> for Commands {
    fn from(post: Post) -> Self {
        Commands::Post(post)
    }
}

impl Commands {
    fn from_matches(sub_command: (&str, Option<&ArgMatches<'static>>)) -> Result<Self, String> {
        let (name, matches) = sub_command;
        let matches = matches.unwrap();

        match name {
            "post" => {
                let message = matches.value_of("message").unwrap().to_string();
                let images = matches.values_of("image").map(|images| images.map(|image| image.to_string()).collect());
                let tags = match matches.values_of("tag") {
                    Some(tags) => tags.map(|value| value.to_string()).collect(),
                    None => vec![],
                };
                let flags = PostFlags { nsfw: matches.is_present("nsfw") };

                Ok(Post { message, tags, flags, images }.into())
            },
            "batch" => {
                let file = matches.value_of("file").unwrap();
                let file = io::read_file_to_string(file)?;
                let file: Batch = toml::from_str(&file).format_err("Invalid config file!")?;
                Ok(file.into())
            },
            "env" => Ok(match matches.subcommand() {
                ("config", _) => EnvCommand::Config.into(),
                _ => unimplemented!(),
            }),
            _ => unimplemented!(),
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
        } else {
            Some(Flags { gab, twitter, minds })
        }
    }
}

#[derive(Debug)]
pub struct Args {
    /// Command to execute
    pub command: Commands,
    pub flags: Flags,
}

impl Args {
    pub fn new(platforms: Platforms) -> Result<Self, String> {
        let matches = parser().get_matches();
        let command = Commands::from_matches(matches.subcommand())?;
        let flags = Flags::from_matches(&matches).unwrap_or(platforms);

        Ok(Args { command, flags })
    }
}
