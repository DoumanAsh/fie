use yukikaze::client::config::Config;
pub use yukikaze::client::request::multipart;
pub use yukikaze::client::request::Builder;
pub use yukikaze::client::Request;
pub use yukikaze::futures::{self, future, Future, IntoFuture};
pub use yukikaze::header;
pub use yukikaze::http::Method;
pub use yukikaze::mime::Mime;
pub use yukikaze::rt::{AutoClient, AutoRuntime, GlobalClient};

use crate::config::Settings;

use std::time::Duration;

static mut TIMEOUT: u64 = 5;

struct Conf;

impl Config for Conf {
    fn timeout() -> Duration {
        get_timeout()
    }
}

pub struct HttpRuntime {
    pub tokio: yukikaze::rt::tokio::Guard,
    pub http: GlobalClient,
}

pub fn init(settings: &Settings) -> HttpRuntime {
    unsafe {
        TIMEOUT = settings.timeout;
    }

    HttpRuntime {
        tokio: yukikaze::rt::tokio::init(),
        http: GlobalClient::with_config::<Conf>(),
    }
}

pub fn get_timeout() -> Duration {
    unsafe { Duration::from_secs(TIMEOUT) }
}
