# Fie

[![Build status](https://ci.appveyor.com/api/projects/status/oc937oppd38x1y4y/branch/master?svg=true)](https://ci.appveyor.com/project/DoumanAsh/fie/branch/master)
[![Build Status](https://travis-ci.org/DoumanAsh/fie.svg?branch=master)](https://travis-ci.org/DoumanAsh/fie)
[![Crates.io](https://img.shields.io/crates/v/fie.svg)](https://crates.io/crates/fie)

Small and cute social media CLI.

![Icon](icon.jpg)

## Download links

* Windows [32bit](https://github.com/DoumanAsh/fie/releases/download/0.9.1/fie-0.9.1-i686-pc-windows-msvc.zip)
* Windows [64bit](https://github.com/DoumanAsh/fie/releases/download/0.9.1/fie-0.9.1-x86_64-pc-windows-msvc.zip)
* Linux [64bit](https://github.com/DoumanAsh/fie/releases/download/0.9.1/fie-0.9.1-x86_64-unknown-linux-gnu.zip)
* OSX [64bit](https://github.com/DoumanAsh/fie/releases/download/0.9.1/fie-0.9.1-x86_64-apple-darwin.zip)

## Supported social platforms:

* Gab (through unofficial API so may break);
* Minds.com (through kinda official API);
* Twitter. Using official API.

## Configuration

Configuration file is placed in the same directory as executable.

See [documentation](docs/configuration.md) on how to setup social medias

Use [example](fie.toml) as reference.

## Usage

```
Small and cute twitter app.

USAGE:
    fie.exe [FLAGS] [SUBCOMMAND]

FLAGS:
        --gab        Use gab.ai. By default all social medias are used unless flag is specified.
    -h, --help       Prints help information
        --minds      Use Minds.com. By default all social medias are used unless flag is specified.
        --twitter    Use Twitter. By default all social medias are used unless flag is specified.
    -V, --version    Prints version information

SUBCOMMANDS:
    env     Prints information about app environment.
    help    Prints this message or the help of the given subcommand(s)
    post    Creates new tweet.
```

### post

Uses to post content on social platforms.
Using `-t` you can specify hashtags which will be appended as last line of content.

```
Creates new tweet

USAGE:
    fie.exe post [OPTIONS] <message>

FLAGS:
    -h, --help    Prints help information

OPTIONS:
    -i <image>...        Adds image to post. Normally up to 4.
    -t <tag>...          Adds hashtag at the last line of post.

ARGS:
    <message>    Message content
```

### batch

```
Load CLI arguments from file and runs it.

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
Prints information about app environment.

USAGE:
    fie.exe env [SUBCOMMAND]

FLAGS:
    -h, --help    Prints help information

SUBCOMMANDS:
    config    Prints path to config file.
    help      Prints this message or the help of the given subcommand(s)
```
