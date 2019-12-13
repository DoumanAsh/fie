# Fie

[![Build status](https://ci.appveyor.com/api/projects/status/oc937oppd38x1y4y/branch/master?svg=true)](https://ci.appveyor.com/project/DoumanAsh/fie/branch/master)
[![Build Status](https://travis-ci.org/DoumanAsh/fie.svg?branch=master)](https://travis-ci.org/DoumanAsh/fie)
[![Crates.io](https://img.shields.io/crates/v/fie.svg)](https://crates.io/crates/fie)
[![Dependency status](https://deps.rs/crate/fie/0.16.0/status.svg)](https://deps.rs/crate/fie)

Small and cute social media CLI.

![Icon](icon.jpg)

## Installation

### Download links

* Windows [32bit](https://github.com/DoumanAsh/fie/releases/download/0.16.0/fie-0.16.0-i686-pc-windows-msvc.zip)
* Windows [64bit](https://github.com/DoumanAsh/fie/releases/download/0.16.0/fie-0.16.0-x86_64-pc-windows-msvc.zip)
* Linux [64bit](https://github.com/DoumanAsh/fie/releases/download/0.16.0/fie-0.16.0-x86_64-unknown-linux-gnu.zip)
* OSX [64bit](https://github.com/DoumanAsh/fie/releases/download/0.16.0/fie-0.16.0-x86_64-apple-darwin.zip)

### Cargo

In order to install CLI utility you need to enable feature `cli`
In addition to that following environment variables are used optionally:

- Twitter Consumer Token (requires both to present for it to be used):
    - `FIE_TWITTER_CONSUMER_KEY` - Builtin Consumer key for twitter API;
    - `FIE_TWITTER_CONSUMER_SECRET` - Builtin Consumer secret for twitter API;

## Supported social platforms:

* Twitter. Using official API.
* Gab. Using official mastodon like API (Note that it is not clear if their fork will change API or not).
* Mastodon. Using official API.
* Minds. Using semi-official API.

## Configuration

Configuration file is placed in the same directory as executable.

See [documentation](docs/configuration.md) on how to setup social medias

Use [example](fie.toml) as reference.

## Usage

```
Small and cute social media utility.

USAGE:
    fie.exe [FLAGS] <SUBCOMMAND>

FLAGS:
    -g, --gab         Use gab.ai. By default all social medias are used unless flag is specified.
    -h, --help        Prints help information
    -m, --mastodon    Use mastodon. By default all social medias are used unless flag is specified.
        --minds       Use minds. By default all social medias are used unless flag is specified.
    -t, --twitter     Use twitter. By default all social medias are used unless flag is specified.
    -V, --version     Prints version information

SUBCOMMANDS:
    auth     Allows to perform authorization with social media.
    batch    Load CLI arguments from file and runs it.
    env      Prints information about app environment.
    help     Prints this message or the help of the given subcommand(s)
    post     Creates new post.
```

### post

Uses to post content on social platforms.
Using `-t` you can specify hashtags which will be appended as last line of content.

```
Creates new post.

USAGE:
    fie.exe post [FLAGS] [OPTIONS] <message>

FLAGS:
    -h, --help    Prints help information
    -n, --nsfw    Whether post is NSFW or not.

OPTIONS:
    -i, --image <images>...    Adds image to post. Normally up to 4.
    -t, --tag <tags>...        Adds hashtag at the last line of post.

ARGS:
    <message>    Message content
```

### batch

Load CLI arguments from file and runs it.

```
USAGE:
    fie.exe batch <file>

FLAGS:
    -h, --help    Prints help information

ARGS:
    <file>    TOML file that describes CLI arguments.
```

File examples:
* [Post](fie_post.toml)

### env

Prints information about app's environment.

```
USAGE:
    fie.exe env <SUBCOMMAND>

FLAGS:
    -h, --help    Prints help information

SUBCOMMANDS:
    config    Prints path to config file.
    help      Prints this message or the help of the given subcommand(s)
```

### auth

Allows to perform user authorization using social media API.
Currently available authorizations:

- Twitter PIN based auth. Interactive dialogue will prompt you to follow link and authorize fie.

```
USAGE:
    fie.exe auth <SUBCOMMAND>

FLAGS:
    -h, --help    Prints help information

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    twitter    Performs authorization with twitter
```
