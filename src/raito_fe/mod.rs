pub mod api_controller;
pub mod renderer;
pub mod routes;

use env;
use mastodon_api;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use std::collections::hash_map::IntoIter;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

pub static mut BYPASS_API: &'static bool = &false;
pub static mut MASTODON_API_BASE_URI: &'static str = "127.0.0.1";

pub struct Authentication {
    pub account: Option<mastodon_api::Account>,
    pub token: Option<String>,
}

#[derive(FromForm)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct LocalConfiguration(HashMap<String, String>);

impl<'a, 'r> FromRequest<'a, 'r> for Authentication {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Authentication, ()> {
        match request.cookies().get_private("oauth_token") {
            Some(token) => {
                match mastodon_api::controller::account_by_oauth_token(token.value().to_string()) {
                    Ok(mastodon_api_account) => Outcome::Success(Authentication {
                        account: Some(mastodon_api_account),
                        token: Some(token.value().to_string()),
                    }),
                    Err(_) => Outcome::Success(Authentication {
                        account: None,
                        token: None,
                    }),
                }
            }
            None => Outcome::Success(Authentication {
                account: None,
                token: None,
            }),
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for LocalConfiguration {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<LocalConfiguration, ()> {
        let mut new_config = HashMap::<String, String>::new();
        new_config.insert("javascript_enabled".to_string(), "false".to_string());
        new_config.insert("mastodon_api_base_uri".to_string(), unsafe {
            if BYPASS_API == &true {
                format!(
                    "{base_scheme}://{base_domain}",
                    base_scheme = env::get_value(String::from("endpoint.base_scheme")),
                    base_domain = env::get_value(String::from("endpoint.base_domain"))
                )
            } else {
                MASTODON_API_BASE_URI.to_string()
            }
        });
        new_config.insert("minimalmode_enabled".to_string(), "false".to_string());
        Outcome::Success(LocalConfiguration(new_config))
    }
}

impl IntoIterator for LocalConfiguration {
    type Item = (String, String);
    type IntoIter = IntoIter<String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub fn get_routes() -> Vec<rocket::Route> {
    routes![
        routes::about,
        routes::account,
        routes::account_follow,
        routes::account_unfollow,
        routes::actor,
        routes::global_timeline,
        routes::home_timeline,
        routes::index,
        routes::login,
        routes::login_post,
        routes::object,
        routes::public_timeline,
        routes::register,
        routes::settings,
        routes::status_compose,
        routes::status_draft,
        routes::view_status
    ]
}

// This should only be a temporary solution, as it ought to be replace by actual theming
pub fn get_stylesheet() -> String {
    let mut style = File::open("static/raito_fe/themes/raito_light.css").expect("theme not found");

    let mut contents = String::new();
    style
        .read_to_string(&mut contents)
        .expect("something went wrong reading the file");
    return contents;
}
