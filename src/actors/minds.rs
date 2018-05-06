//! Actors to access Minds API

extern crate futures;
extern crate actix;
extern crate actix_web;

use self::futures::{
    future,
    Future
};
use self::actix::prelude::*;
use self::actix_web::HttpMessage;
use self::actix_web::client::ClientRequest;

use ::config;
use ::misc::{
    ClientRequestBuilderExt,
    ClientRequestExt
};
use super::messages::{
    PostMessage,
    ResultMessage,
    UploadImage,
    ResultImage,
    PostFlags
};


mod payload {
    use super::PostFlags;

    #[derive(Serialize, Debug)]
    pub struct Auth {
        grant_type: &'static str,
        client_id: &'static str,
        client_secret: &'static str,
        username: String,
        password: String,
    }

    impl Auth {
        pub fn new(username: String, password: String) -> Self {
            Auth {
                grant_type: "password",
                client_id: "",
                client_secret: "",
                username,
                password
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Oauth2 {
        pub access_token: String,
        pub user_id: String,
        pub refresh_token: String
    }

    #[derive(Serialize, Debug)]
    pub struct Post<'a> {
        wire_threshold: Option<String>,
        message: &'a str,
        is_rich: u8,
        title: Option<String>,
        description: Option<String>,
        thumbnail: Option<String>,
        url: Option<String>,
        attachment_guid: Option<String>,
        pub mature: u8,
        access_id: u8
    }

    impl<'a> Post<'a> {
        pub fn new(message: &'a str, attachment_guid: Option<String>, flags: &PostFlags) -> Self {
            Post {
                wire_threshold: None,
                message,
                is_rich: 0,
                title: None,
                description: None,
                thumbnail: None,
                url: None,
                attachment_guid,
                mature: flags.nsfw as u8,
                access_id: 2
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct UploadResponse {
        pub guid: String
    }
}

pub struct Minds {
    config: Option<config::Minds>,
    oauth2: Option<payload::Oauth2>
}

impl Actor for Minds {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        const OAUTH2_URL: &'static str = "https://www.minds.com/oauth2/token";
        let config = self.config.take().unwrap();
        let mut req = ClientRequest::post(OAUTH2_URL);
        let req = req.set_default_headers()
                     .json(payload::Auth::new(config.username, config.password))
                     .map_err(|error| format!("Unable to serialize oauth2 request. Error: {}", error));

        let req = match req {
            Ok(req) => req,
            Err(error) => {
                eprintln!("Unable to serialize oauth2 request. Error: {}", error);
                return ctx.stop();
            }
        };

        req.send_ext().into_actor(self).map_err(|error, _act, ctx| {
            eprintln!("Minds oauth2 error: {}", error);
            ctx.stop();
        }).and_then(|rsp, act, _ctx| {
            rsp.json().into_actor(act)
                      .map(|oauth2, act, _ctx| act.oauth2 = Some(oauth2))
                      .map_err(|error, _act, ctx| {
                          eprintln!("Minds oauth2 parse error: {}", error);
                          ctx.stop()
                      })
        }).wait(ctx);
    }
}

impl Minds {
    pub fn new(config: config::Minds) -> Self {
        Self {
            config: Some(config),
            oauth2: None
        }
    }
}

impl Handler<UploadImage> for Minds {
    type Result = ResponseFuture<ResultImage, String>;

    fn handle(&mut self, msg: UploadImage, _: &mut Self::Context) -> Self::Result {
        const IMAGES_URL: &'static str = "https://www.minds.com/api/v1/media";

        let access_token = match self.oauth2.as_ref() {
            Some(ref oauth2) => &oauth2.access_token,
            None => return Box::new(future::err("Unable to send Minds request".to_string())),
        };

        let name = &msg.0.name;
        let mime = &msg.0.mime;
        let data = &msg.0.mmap[..];

        let mut req = ClientRequest::post(IMAGES_URL);

        let req = match req.set_default_headers().auth_bearer(&access_token).set_multipart_body(&name, &mime, &data) {
            Ok(req) => req,
            Err(error) => return Box::new(future::err(error))
        };

        let req = req.send_ext().map_err(|error| format!("Minds upload error: {}", error))
                     .and_then(|response| match response.status().is_success() {
                         true => Ok(response),
                         false => Err(format!("Minds server returned error code {}", response.status())),
                     })
                     .and_then(|response| response.json().map_err(|error| format!("Minds upload reading error: {}", error)))
                     .map(|response: payload::UploadResponse| ResultImage::Guid(response.guid));

        Box::new(req)
    }
}

#[derive(Deserialize, Debug)]
pub struct PostResponse {
    pub guid: String
}

impl Handler<PostMessage> for Minds {
    type Result = ResponseFuture<ResultMessage, String>;

    fn handle(&mut self, msg: PostMessage, _: &mut Self::Context) -> Self::Result {
        const POST_URL: &'static str = "https://www.minds.com/api/v1/newsfeed";

        let access_token = match self.oauth2.as_ref() {
            Some(ref oauth2) => &oauth2.access_token,
            None => return Box::new(future::err("Unable to send Minds request".to_string())),
        };

        let PostMessage{flags, content, images} = msg;

        let mut req = ClientRequest::post(POST_URL);

        let images = match images {
            Some(mut images) => images.drain(..).next().map(|image| image.guid()),
            None => None
        };

        let req = req.set_default_headers()
                     .auth_bearer(access_token)
                     .json(payload::Post::new(&content, images, &flags));

        let req = match req {
            Ok(req) => req,
            Err(error) => return Box::new(future::err(format!("Minds post actix error: {}", error)))
        };

        let req = req.send_ext().map_err(|error| format!("Minds post error: {}", error))
                     .and_then(|response| match response.status().is_success() {
                         true => Ok(response),
                         false => Err(format!("Minds server returned error code {}", response.status())),
                     })
                     .and_then(|response| response.json::<PostResponse>().map_err(|error| format!("Minds post error: {}", error)))
                     .map(|response| ResultMessage::Guid(response.guid));

        Box::new(req)
    }
}
