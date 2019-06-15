# Configuration

## Location

Fie looks up following locations:

- `<directory with executable>/fie.toml`
- `<HOME>/.fie/fie.toml`

If first is missing it uses the home's config.
If both are missing then error happens

## Gab

Just provide your password and login

```toml
[api.gab]
username = "username"
password = "password"
```

## Minds

Just provide your password and login

```toml
[api.gab]
username = "username"
password = "password"
```

## Twitter

```toml
[api.twitter.consumer]
key = "key"
secret = "secret"
```

`Your Access Token` has `Access Token` and `Access Token Secret`
Put it in section below:

```toml
[api.twitter.access]
key = "token"
secret = "secret"
```

### Using own application

Go to [app page](https://apps.twitter.com/) and create new app for yourself.

After that go to section `Keys and Access Tokens` to retrieve configuration:

`Application Settings` has `Consumer Key` and `Consumer Secret`
Put it in section below:

### Using fie builtin tokens

In this case fie must have been built with following environment variables:

- `FIE_TWITTER_CONSUMER_KEY` - contains consumer key of application.
- `FIE_TWITTER_CONSUMER_SECRET` - contains consumer secret of application.

Provided download links will contain `fie` own consumer token.
Therefore `api.twitter.consumer` can be omitted

In this case you can use command `fie auth twitter` in order to get `api.twitter.access`
After successfully following interactive instructions, the `api.twitter.access` configuration will be printed in stdout.
This should replace whatever you already have in your configuration.

Use `fie env config` to find configuration file location.

## Mastodon

You need to provide host name of the Mastodon instance.

**Note:** that it should be without `http` prefix

Access token can be granted by creating own application via `Preferences->Developement->New Application`

```toml
[api.mastodon]
host = "pawoo.net"
access_token = ""
```
