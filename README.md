# Fie

[![Build status](https://ci.appveyor.com/api/projects/status/oc937oppd38x1y4y/branch/master?svg=true)](https://ci.appveyor.com/project/DoumanAsh/fie/branch/master)
[![Build Status](https://travis-ci.org/DoumanAsh/fie.svg?branch=master)](https://travis-ci.org/DoumanAsh/fie)

Small and cute social media CLI.

![Icon](icon.jpg)

## Supported platforms:

* Gab (through unofficial API so may break);
* Minds.com (through kinda official API);
* Twitter

## Configuration

Configuration file is placed in the same directory as executable.

Use [example](fie.toml) as reference.

## Usage

```
fie 0.2.0
Douman <douman@gmx.se>

Small and cute twitter app.

USAGE:
    fie.exe [FLAGS] [SUBCOMMAND]

FLAGS:
        --gab        Use gab.ai. By default all social medias are used unless flag is specified.
    -h, --help       Prints help information
        --twitter    Use Twitter. By default all social medias are used unless flag is specified.
    -V, --version    Prints version information

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand(s)
    post    Creates new tweet
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


