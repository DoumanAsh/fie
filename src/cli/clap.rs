pub use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

#[inline(always)]
/// Shortcut to create CLI argument
pub fn arg(name: &str) -> Arg {
    Arg::with_name(name)
}

#[inline(always)]
/// Shortcut to create CLI option/flag
pub fn flag(name: &str) -> Arg {
    arg(name).long(name)
}

const NAME: &'static str = env!("CARGO_PKG_NAME");
const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const ABOUT: &'static str = "
Small and cute twitter app.";

#[inline(always)]
fn post_command() -> App<'static, 'static> {
    SubCommand::with_name("post")
        .about("Creates new tweet.")
        .arg(arg("message").required(true).help("Message content"))
        .arg(
            arg("tag")
                .short("t")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true)
                .help("Adds hashtag at the last line of post."),
        ).arg(arg("image").short("i").multiple(true).takes_value(true).help("Adds image to post. Normally up to 4."))
        .arg(flag("nsfw").help("Whether post is NSFW or not."))
}

#[inline(always)]
fn env_config_command() -> App<'static, 'static> {
    SubCommand::with_name("config").about("Prints path to config file.")
}

#[inline(always)]
fn env_command() -> App<'static, 'static> {
    SubCommand::with_name("env")
        .about("Prints information about app environment.")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(env_config_command())
}

#[inline(always)]
fn batch_command() -> App<'static, 'static> {
    SubCommand::with_name("batch")
        .about("Load CLI arguments from file and runs it.")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(arg("file").required(true).help("TOML file that describes CLI arguments."))
}

pub fn parser() -> App<'static, 'static> {
    App::new(NAME)
        .about(ABOUT)
        .author(AUTHOR)
        .version(VERSION)
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(flag("gab").help("Use gab.ai. By default all social medias are used unless flag is specified."))
        .arg(flag("twitter").help("Use Twitter. By default all social medias are used unless flag is specified."))
        .arg(flag("minds").help("Use Minds.com. By default all social medias are used unless flag is specified."))
        .arg(flag("mastodon").help("Use Mastodon. By default all social medias are used unless flag is specified."))
        .subcommand(post_command())
        .subcommand(env_command())
        .subcommand(batch_command())
}
