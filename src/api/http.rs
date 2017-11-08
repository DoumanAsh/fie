pub use ::hyper::{Client, Request, Method, Response, StatusCode};
pub use ::hyper::header::{ContentType, ContentLength, Authorization, Bearer};
pub use ::hyper::client::{HttpConnector, FutureResponse};
pub use ::hyper_tls::{HttpsConnector};
pub use ::hyper::mime;

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
