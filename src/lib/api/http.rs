//!Http runtime

use yukikaze::client::config::Config;
pub use yukikaze::client::request::multipart;
pub use yukikaze::client::request::Builder;
pub use yukikaze::client::Request;
pub use yukikaze::futures::{self, future, Future, IntoFuture};
pub use yukikaze::header;
pub use yukikaze::http::{Uri, Method};
pub use yukikaze::mime::Mime;
pub use yukikaze::rt::{AutoClient, GlobalClient};
pub use tokio_global::AutoRuntime;

use crate::config::Settings;

use std::time::Duration;

static mut TIMEOUT: u64 = 5;

struct Conf;

impl Config for Conf {
    fn timeout() -> Duration {
        get_timeout()
    }
}

///Http runtime used by fie
pub struct HttpRuntime {
    _tokio: tokio_global::single::Runtime,
    _http: GlobalClient,
}

///Initializes instance of runtime
pub fn init(settings: &Settings) -> HttpRuntime {
    unsafe {
        TIMEOUT = settings.timeout;
    }

    HttpRuntime {
        _tokio: tokio_global::single::init(),
        _http: GlobalClient::with_config::<Conf>(),
    }
}

///Gets currently set timeout value
pub fn get_timeout() -> Duration {
    unsafe { Duration::from_secs(TIMEOUT) }
}
