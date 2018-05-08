extern crate actix_web;
extern crate mime_guess;

use std::fmt;
use std::time;

use self::actix_web::client::{ClientRequest, ClientRequestBuilder, SendRequest};
use self::actix_web::http;
use self::mime_guess::Mime;

/// Extension to std Result.
pub trait ResultExt<T, E> {
    /// Formats error to string.
    fn format_err(self, prefix: &str) -> Result<T, String>;
}

impl<T, E: fmt::Display> ResultExt<T, E> for Result<T, E> {
    fn format_err(self, prefix: &str) -> Result<T, String> {
        self.map_err(|error| format!("{}. Error: {}", prefix, error))
    }
}

///Number of seconds to wait for connection
const CONN_TIMEOUT_S: u64 = 5;
///Number of seconds to wait for response
const WAIT_TIMEOUT_S: u64 = 300;

pub trait ClientRequestExt {
    fn send_ext(self) -> SendRequest;
}
impl ClientRequestExt for ClientRequest {
    fn send_ext(self) -> SendRequest {
        self.send()
            .timeout(time::Duration::new(WAIT_TIMEOUT_S, 0))
            .conn_timeout(time::Duration::new(CONN_TIMEOUT_S, 0))
            .wait_timeout(time::Duration::new(WAIT_TIMEOUT_S, 0))
    }
}

pub trait ClientRequestBuilderExt {
    fn set_default_headers(&mut self) -> &mut Self;
    fn set_multipart_body(&mut self, file_name: &str, mime: &Mime, data: &[u8]) -> Result<ClientRequest, String>;
    fn auth_bearer(&mut self, token: &str) -> &mut Self;
}

impl ClientRequestBuilderExt for ClientRequestBuilder {
    fn set_default_headers(&mut self) -> &mut Self {
        self.header(
            http::header::USER_AGENT,
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/49.0.2623.87 Safari/537.36",
        ).header(http::header::ACCEPT_ENCODING, "gzip, deflate")
    }

    fn set_multipart_body(&mut self, file_name: &str, mime: &Mime, data: &[u8]) -> Result<ClientRequest, String> {
        const BOUNDARY: &'static str = "-fie";
        let mut body = Vec::with_capacity(data.len());
        body.extend(format!("\r\n--{}\r\n", BOUNDARY).as_bytes().iter());
        body.extend(format!("Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n", file_name).as_bytes().iter());
        body.extend(format!("Content-Type: {}\r\n\r\n", mime).as_bytes().iter());
        body.extend(data.iter());
        body.extend(format!("\r\n--{}--\r\n", BOUNDARY).as_bytes().iter());
        let len = body.len() as u64;

        let content_type = format!("multipart/form-data; boundary={}", BOUNDARY);
        self.header(http::header::CONTENT_TYPE, content_type)
            .content_length(len)
            .body(body)
            .format_err("Actix request body failure")
    }

    fn auth_bearer(&mut self, token: &str) -> &mut Self {
        self.header(http::header::AUTHORIZATION, format!("Bearer {}", token))
    }
}
