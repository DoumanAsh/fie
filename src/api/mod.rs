use crate::cli;
use crate::config;
use crate::io;
use crate::http::{future, AutoRuntime, HttpRuntime, Future};

mod gab;
mod minds;
mod twitter;

pub struct API {
    twitter: Option<twitter::Twitter>,
    gab: Option<gab::Gab>,
    minds: Option<minds::Minds>,
    pub settings: config::Settings,
    _http_guard: HttpRuntime,
}

impl API {
    pub fn new(settings: config::Settings) -> Self {
        let _http_guard = crate::http::init(&settings);

        Self {
            twitter: None,
            gab: None,
            minds: None,
            settings,
            _http_guard,
        }
    }

    pub fn start_twitter_if(mut self, cond: bool, twitter: config::Twitter) -> Self {
        if cond {
            self.twitter = twitter::Twitter::new(twitter);
            if self.twitter.is_none() {
                eprintln!("Twitter: Unable to authorize");
            }
        }

        self
    }

    pub fn start_gab_if(mut self, cond: bool, gab: config::Gab) -> Self {
        if cond {
            self.gab = gab::Gab::new(gab);
            if self.gab.is_none() {
                eprintln!("Gab: Unable to get JWT token");
            }
        }

        self
    }

    pub fn start_minds_if(mut self, cond: bool, minds: config::Minds) -> Self {
        if cond {
            self.minds = minds::Minds::new(minds);
            if self.minds.is_none() {
                eprintln!("Minds: Unable to authorize through Oauth2");
            }
        }

        self
    }

    fn join_hash_tags(tags: Vec<String>) -> String {
        let mut result = String::new();

        for tag in tags {
            result.push_str(&format!("#{} ", tag));
        }

        // remove last white space
        let _ = result.pop();

        result
    }

    pub fn send(&self, post: cli::Post) {
        let cli::Post { message, tags, flags, images } = post;

        let message = if tags.len() > 0 {
            match message.as_str() {
                "" => Self::join_hash_tags(tags),
                message => format!("{}\n{}", message, Self::join_hash_tags(tags)),
            }
        } else {
            message
        };

        let message = message.as_str();
        let flags = &flags;

        let mut jobs: Vec<Box<Future<Item = (), Error = ()>>> = vec![];

        match images {
            Some(ref images) if images.len() > 0 => {
                let images = {
                    let mut result = vec![];
                    for image in images {
                        match io::Image::open(image) {
                            Ok(image) => result.push(image),
                            Err(error) => {
                                eprintln!("Error opening image '{}': {}", image, error);
                                return;
                            },
                        };
                    }
                    result
                };

                if let Some(ref twitter) = self.twitter {
                    let mut uploads = vec![];
                    for image in images.iter() {
                        let upload = twitter.upload_image(&image.name, &image.mime, &image.mmap[..]);
                        uploads.push(Box::new(upload));
                    }

                    let uploads = future::join_all(uploads)
                        .and_then(move |uploads| twitter.post(&message, &uploads, &flags))
                        .or_else(|_| Ok(()));

                    jobs.push(Box::new(uploads));
                }

                if let Some(ref gab) = self.gab {
                    let mut uploads = vec![];
                    for image in images.iter() {
                        let upload = gab.upload_image(&image.name, &image.mime, &image.mmap[..]);
                        uploads.push(Box::new(upload));
                    }

                    let uploads = future::join_all(uploads).and_then(move |uploads| gab.post(&message, &uploads, &flags)).or_else(|_| Ok(()));

                    jobs.push(Box::new(uploads));
                }

                if let Some(ref minds) = self.minds {
                    match images.len() {
                        1 => (),
                        0 => unreachable!(),
                        _ => eprintln!("Minds.com accepts only one attachment, only first image will be attached"),
                    }

                    let image = unsafe { images.get_unchecked(0).clone() };
                    let image = minds.upload_image(&image.name, &image.mime, &image.mmap[..]);
                    let upload = image.and_then(move |image| minds.post(&message, Some(image), &flags)).or_else(|_| Ok(()));

                    jobs.push(Box::new(upload));
                }
            },
            _ => {
                if let Some(ref twitter) = self.twitter {
                    let post = twitter.post(&message, &[], &flags).or_else(|_| Ok(()));
                    jobs.push(Box::new(post));
                }
                if let Some(ref gab) = self.gab {
                    let post = gab.post(&message, &[], &flags).or_else(|_| Ok(()));
                    jobs.push(Box::new(post));
                }
                if let Some(ref minds) = self.minds {
                    let post = minds.post(&message, None, &flags).or_else(|_| Ok(()));
                    jobs.push(Box::new(post));
                }
            },
        }

        let _ = future::join_all(jobs).finish();
    }
}
