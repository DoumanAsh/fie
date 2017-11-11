extern crate cookie;

pub use ::hyper::{Client, Request, Method, Response, StatusCode};
pub use ::hyper::header::{Accept, ContentType, ContentLength, Authorization, Bearer, Cookie, SetCookie};
pub use ::hyper::client::{HttpConnector, FutureResponse};
pub use ::hyper_tls::{HttpsConnector};
pub use ::hyper::mime;
use ::tokio_core::reactor::{
    Handle
};

pub trait MultipartBody {
    fn set_multipart_body(&mut self, boundary: &str, file_name: &str, mime: &mime::Mime, data: &[u8]);
}

impl MultipartBody for Request {
    fn set_multipart_body(&mut self, boundary: &str, file_name: &str, mime: &mime::Mime, data: &[u8]) {
        let mut body = Vec::with_capacity(data.len());
        body.extend(format!("\r\n--{}\r\n", boundary).as_bytes().iter());
        body.extend(format!("Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n", file_name).as_bytes().iter());
        body.extend(format!("Content-Type: {}\r\n\r\n", mime).as_bytes().iter());
        body.extend(data.iter());
        body.extend(format!("\r\n--{}--\r\n", boundary).as_bytes().iter());
        let len = body.len() as u64;

        let content_type = format!("multipart/form-data; boundary={}", boundary).parse().unwrap();
        self.headers_mut().set(ContentType(content_type));
        self.headers_mut().set(ContentLength(len));
        self.set_body(body);
    }
}

pub type HttpClient = Client<HttpsConnector<HttpConnector>>;
pub fn create_client(handle: &Handle) -> HttpClient {
    Client::configure().keep_alive(true)
                       .connector(HttpsConnector::new(4, handle).unwrap())
                       .build(handle)
}

#[allow(dead_code)]
pub fn from_set_to_cookie(set_cookie: Option<&SetCookie>) -> Cookie {
    let mut result = Cookie::new();

    if let Some(set_cookie) = set_cookie {
        for setter in set_cookie.iter() {
            let cookie = cookie::Cookie::parse(setter.to_string()).expect("Invalid cookie");
            result.set(cookie.name().to_string(), cookie.value().to_string());
        }
    }

    result
}
