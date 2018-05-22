//! Twitter's data primitives.
extern crate actix_web;
extern crate data_encoding;
extern crate percent_encoding;
extern crate rand;
extern crate ring;

use self::actix_web::http::Method;
use self::percent_encoding::{utf8_percent_encode, EncodeSet};
use std::collections::HashMap;

#[derive(Copy, Clone)]
struct TwitterEncodeSet;

impl EncodeSet for TwitterEncodeSet {
    fn contains(&self, byte: u8) -> bool {
        match byte {
            b'a'...b'z' | b'A'...b'Z' | b'0'...b'9' | b'-' | b'.' | b'_' | b'~' => false,
            _ => true,
        }
    }
}

/// Encodes the given string slice for transmission to Twitter.
fn percent_encode(src: &str) -> impl Iterator<Item = &str> {
    utf8_percent_encode(src, TwitterEncodeSet)
}

use config;

pub struct Oauth {
    /// Contains percent encoded consumer's key
    consumer_key: String,
    /// Contains percent encoded consumer and access token secrets
    ///
    /// It is used as seeding value for hmac generation of signature.
    signature_key: String,
    oauth_signature_method: &'static str,
    /// Contains percent encoded access token's key
    oauth_token: String,
    oauth_version: &'static str,
}

impl Oauth {
    pub fn new(config: config::Twitter) -> Self {
        let consumer_key = percent_encode(&config.consumer.key).collect();
        let oauth_token = percent_encode(&config.access.key).collect();
        let oauth_version = "1.0";
        let oauth_signature_method = "HMAC-SHA1";
        let signature_key = {
            let mut result = String::new();
            for ch in percent_encode(&config.consumer.secret) {
                result.push_str(ch);
            }
            result.push_str("&");
            for ch in percent_encode(&config.access.secret) {
                result.push_str(ch);
            }
            result
        };

        Self {
            consumer_key,
            oauth_token,
            oauth_version,
            oauth_signature_method,
            signature_key,
        }
    }

    /// Returns Authorization header value
    ///
    /// The important thing here is signature which is
    /// derived from method, uri and payload params.
    pub fn gen_auth(&self, method: &Method, uri: &str, params: HashMap<&str, &str>) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = Self::nonce();

        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(dur) => dur,
            Err(err) => err.duration(),
        }.as_secs();
        let timestamp = &format!("{}", timestamp);

        let signature_params = {
            let mut query_params = params;
            query_params.insert("oauth_consumer_key", self.consumer_key.as_str());
            query_params.insert("oauth_nonce", nonce.as_str());
            query_params.insert("oauth_signature_method", self.oauth_signature_method);
            query_params.insert("oauth_timestamp", timestamp.as_str());
            query_params.insert("oauth_version", self.oauth_version);
            query_params.insert("oauth_token", self.oauth_token.as_str());

            let mut query = query_params
                .iter()
                .map(|(k, v)| format!("{}={}", percent_encode(k).collect::<String>(), percent_encode(v).collect::<String>()))
                .collect::<Vec<_>>();
            query.sort();
            query.join("&")
        };

        let signature = self.signature(method, uri, Some(signature_params));
        let mut header_value = String::new();
        header_value.push_str("OAuth");
        header_value.push_str(" oauth_consumer_key=\"");
        header_value.push_str(self.consumer_key.as_str());
        header_value.push_str("\", oauth_nonce=\"");
        header_value.push_str(nonce.as_str());
        header_value.push_str("\", oauth_signature=\"");
        for ch in percent_encode(&signature) {
            header_value.push_str(ch);
        }
        header_value.push_str("\", oauth_signature_method=\"");
        header_value.push_str(self.oauth_signature_method);
        header_value.push_str("\", oauth_timestamp=\"");
        header_value.push_str(timestamp.as_str());
        header_value.push_str("\", oauth_token=\"");
        header_value.push_str(self.oauth_token.as_str());
        header_value.push_str("\", oauth_version=\"");
        header_value.push_str(self.oauth_version);
        header_value.push_str("\"");

        header_value
    }

    /// Generates Authorization's signature based on method, uri and params.
    ///
    /// Parameters are composed into single percent encoded string and
    /// signed using HMAC-SHA1 algorithm
    /// `signature_key` is used as seed to algorithm.
    ///
    /// Return base64 encoded string.
    fn signature(&self, method: &Method, uri: &str, params: Option<String>) -> String {
        use self::data_encoding::BASE64;
        use self::ring::{digest, hmac};

        let key = hmac::SigningKey::new(&digest::SHA1, self.signature_key.as_bytes());
        let signature = {
            let mut result = String::new();
            for ch in percent_encode(method.as_str()) {
                result.push_str(ch);
            }
            result.push_str("&");
            for ch in percent_encode(uri) {
                result.push_str(ch);
            }
            result.push_str("&");
            let params = params.unwrap_or_default();
            for ch in percent_encode(&params) {
                result.push_str(ch);
            }

            result
        };
        let signature = hmac::sign(&key, signature.as_bytes());
        BASE64.encode(signature.as_ref())
    }

    fn nonce() -> String {
        use self::rand::{distributions, thread_rng, Rng};

        thread_rng().sample_iter(&distributions::Alphanumeric).take(32).collect()
    }
}

#[derive(Serialize, Debug)]
pub struct Media {
    pub media_data: String,
}

impl Media {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        use self::data_encoding::BASE64;
        let media_data = BASE64.encode(bytes);
        Self { media_data }
    }
}

#[derive(Deserialize, Debug)]
pub struct MediaResponse {
    pub media_id: u64,
}

#[derive(Serialize, Debug)]
pub struct Tweet {
    pub status: String,
    pub media_ids: Option<String>,
    pub possibly_sensitive: bool,
}

impl Tweet {
    pub fn new(status: String) -> Self {
        Self {
            status,
            media_ids: None,
            possibly_sensitive: false,
        }
    }

    pub fn nsfw(mut self, value: bool) -> Self {
        self.possibly_sensitive = value;
        self
    }

    pub fn media_ids(mut self, ids: &[u64]) -> Self {
        if !ids.is_empty() {
            let ids = ids.iter().map(|id| id.to_string()).collect::<Vec<String>>().join(",");
            self.media_ids = Some(ids);
        }
        self
    }
}

#[derive(Deserialize, Debug)]
pub struct TweetResponse {
    pub id: u64,
}
