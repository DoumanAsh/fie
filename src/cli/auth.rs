use fie::config;
use fie::api;
use fie::api::http::{self, AutoClient, AutoRuntime, Request};
use serde_derive::{Deserialize};

use std::io::{self, Write};
use std::collections::HashMap;

pub fn twitter(mut config: config::Twitter) {
    const REQUEST_TOKEN_URI: &str = "https://api.twitter.com/oauth/request_token";
    const ACCESS_TOKEN_URI: &str = "https://api.twitter.com/oauth/access_token";

    #[derive(Deserialize, Debug)]
    struct RequestTokenRsp {
        oauth_token: String,
        oauth_token_secret: String,
    }

    config.access.key.truncate(0);
    config.access.secret.truncate(0);

    let mut oauth = api::twitter::data::Oauth::new(config);

    let _http = http::init(&Default::default());

    let (auth_params, auth_header) = {
        let mut auth_params = HashMap::new();
        auth_params.insert("oauth_callback", "oob");
        auth_params.insert("x_auth_access_type", "write");
        (auth_params.clone(), oauth.gen_auth(&http::Method::POST, REQUEST_TOKEN_URI, auth_params))
    };

    let req = Request::post(REQUEST_TOKEN_URI).expect("To create request")
                                              .set_header(http::header::AUTHORIZATION, auth_header)
                                              .form(&auth_params)
                                              .expect("To serialize form params")
                                              .send()
                                              .finish();

    let request_token: RequestTokenRsp = match req {
        Ok(response) => match response.is_success() {
            true => match response.text().finish() {
                Ok(response) => match yukikaze::serde_urlencoded::from_str(&response) {
                    Ok(response) => response,
                    Err(error) => {
                        eprintln!("Unable to parse response with request token. Error: {}", error);
                        return;
                    }
                },
                Err(error) => {
                    eprintln!("Failed to read response with requested token. Error: {}", error);
                    return;
                }
            },
            false => {
                eprintln!("Request for token failed with {}", response.status());
                return;
            }
        },
        Err(error) => {
            eprintln!("Failed to request ouath token :( Error: {}", error);
            return;
        }
    };

    println!("Please use following link to authroize fie:\nhttps://api.twitter.com/oauth/authorize?oauth_token={}", request_token.oauth_token);
    println!("Once done please enter PIN...");
    let pin = {
        let mut buffer = String::new();
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        loop {
            buffer.truncate(0);

            let _ = stdout.write_all(b"Print: ");
            let _ = stdout.flush();
            match stdin.read_line(&mut buffer) {
                Ok(_) => (),
                Err(_) => {
                    let _ = stdout.write_all(b"Failed to read input. Try again...\n");
                    continue;
                }
            }

            let pin = buffer.trim();
            match pin.parse::<u32>() {
                Ok(_) => break pin.to_owned(),
                Err(_) => {
                    let _ = stdout.write_all(b"Invalid PIN specified, should contain only digits. Try again...\n");
                    continue;
                }
            }
        }
    };

    oauth.set_oauth_token(&request_token.oauth_token);
    let (auth_params, auth_header) = {
        let mut auth_params = HashMap::new();
        auth_params.insert("oauth_verifier", pin.trim());
        (auth_params.clone(), oauth.gen_auth(&http::Method::POST, ACCESS_TOKEN_URI, auth_params))
    };

    let req = Request::post(ACCESS_TOKEN_URI).expect("To create request")
                                             .set_header(http::header::AUTHORIZATION, auth_header)
                                             .form(&auth_params)
                                             .expect("To serialize form params")
                                             .send()
                                             .finish();

    let access_token: RequestTokenRsp = match req {
        Ok(response) => match response.is_success() {
            true => match response.text().finish() {
                Ok(response) => match yukikaze::serde_urlencoded::from_str(&response) {
                    Ok(response) => response,
                    Err(error) => {
                        eprintln!("Unable to parse response with access token. Error: {}", error);
                        return;
                    }
                },
                Err(error) => {
                    eprintln!("Failed to read response with access token. Error: {}", error);
                    return;
                }
            },
            false => {
                eprintln!("Request for access token failed with {}", response.status());
                return;
            }
        },
        Err(error) => {
            eprintln!("Failed to request access token :( Error: {}", error);
            return;
        }
    };

    println!("Received access token successfully.\nAdd following to your fie configuration file:");
    println!("[api.twitter.access]\nkey = \"{}\"\nsecret = \"{}\"", access_token.oauth_token, access_token.oauth_token_secret);
}
