//!Http runtime

use yukikaze::client::config::Config;
pub use yukikaze::client::Request;
pub use yukikaze::client::request::multipart;
pub use yukikaze::client::request::Builder;
pub use yukikaze::header;
pub use yukikaze::http::{Uri, Method};
pub use yukikaze::mime::Mime;
pub use yukikaze::matsu;

use crate::config::Settings;

use core::time::Duration;
pub use core::future::Future;

static mut TIMEOUT: u64 = 5;

///Yukikaze config
pub struct Conf;

impl Config for Conf {
    type Connector = yukikaze::connector::rustls::HttpsOnlyConnector;
    type Timer = yukikaze::client::config::DefaultTimer;

    fn timeout() -> Duration {
        get_timeout()
    }
}

mod gen {
    use super::Conf;

    yukikaze::declare_global_client!(Conf);
}

pub use gen::GlobalRequest;

///Sets current timeout value;
pub fn set_timeout(settings: &Settings) {
    unsafe {
        TIMEOUT = settings.timeout;
    }
}

///Gets currently set timeout value
pub fn get_timeout() -> Duration {
    unsafe { Duration::from_secs(TIMEOUT) }
}
