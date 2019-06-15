use std::mem;

use structopt::StructOpt;

use fie::config::Platforms;

#[derive(Debug, StructOpt)]
#[structopt(name = "fie", raw(setting = "structopt::clap::AppSettings::ArgRequiredElseHelp"), raw(setting = "structopt::clap::AppSettings::VersionlessSubcommands"))]
pub struct Args {
    #[structopt(flatten)]
    pub flags: Flags,
    #[structopt(subcommand)]
    pub cmd: Command,
}

impl Args {
    #[inline]
    pub fn new(platforms: &mut Platforms) -> Self {
        let args = Self::from_args();

        //Unless user specifies manually, we use configuration defaults
        if args.flags.twitter || args.flags.gab || args.flags.mastodon || args.flags.minds {
            *platforms = unsafe { mem::transmute(args.flags) }
        }

        args
    }
}

#[derive(Debug, Copy, Clone, StructOpt)]
pub struct Flags {
    #[structopt(short = "t", long = "twitter")]
    ///Use twitter. By default all social medias are used unless flag is specified.
    pub twitter: bool,
    #[structopt(short = "g", long = "gab")]
    ///Use gab.ai. By default all social medias are used unless flag is specified.
    pub gab: bool,
    #[structopt(short = "m", long = "mastodon")]
    ///Use mastodon. By default all social medias are used unless flag is specified.
    pub mastodon: bool,
    #[structopt(long = "minds")]
    ///Use minds. By default all social medias are used unless flag is specified.
    pub minds: bool,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "post")]
    ///Creates new post.
    Post(Post),
    #[structopt(name = "env")]
    ///Prints information about app environment.
    Env(Env),
    #[structopt(name = "batch")]
    ///Load CLI arguments from file and runs it.
    Batch(Batch),
    #[structopt(name = "auth")]
    ///Allows to perform authorization with social media.
    Auth(Auth),
}

#[derive(Debug, StructOpt)]
pub struct Post {
    ///Message content
    pub message: String,
    #[structopt(short = "t", long = "tag")]
    ///Adds hashtag at the last line of post.
    pub tags: Vec<String>,
    #[structopt(short = "i", long = "image")]
    ///Adds image to post. Normally up to 4.
    pub images: Vec<String>,
    #[structopt(short = "n", long = "nsfw")]
    ///Whether post is NSFW or not.
    pub nsfw: bool,
}

impl Into<fie::data::Post> for Post {
    fn into(self) -> fie::data::Post {
        let Post { message, tags, images, nsfw } = self;

        fie::data::Post {
            message,
            tags,
            images,
            flags: fie::data::PostFlags {
                nsfw
            }
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct Batch {
    ///TOML file that describes CLI arguments.
    pub file: String,
}

#[derive(Debug, StructOpt)]
pub enum Env {
    #[structopt(name = "config")]
    ///Prints path to config file.
    Config,
}

#[derive(Debug, StructOpt)]
pub enum Auth {
    #[structopt(name = "twitter")]
    ///Performs authorization with twitter
    Twitter,
}
