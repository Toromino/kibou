pub mod api_controller;
pub mod renderer;
pub mod routes;

use database::{self, PooledConnection};
use env;
use mastodon_api;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::State;
use std::collections::hash_map::IntoIter;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

pub static mut BYPASS_API: &'static bool = &false;
pub static mut MASTODON_API_BASE_URI: &'static str = "127.0.0.1";

pub struct Configuration {
    pub account: Option<mastodon_api::Account>,
    pub context: HashMap<String, String>,
    pub token: Option<String>,
}

#[derive(FromForm)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for Configuration {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Configuration, ()> {
        let mut account: Option<mastodon_api::Account> = None;
        let mut context = HashMap::<String, String>::new();
        let mut token: Option<String> = None;

        context.insert("javascript_enabled".to_string(), "false".to_string());
        context.insert("mastodon_api_base_uri".to_string(), unsafe {
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
        context.insert("minimalmode_enabled".to_string(), "false".to_string());
        context.insert("stylesheet".to_string(), get_stylesheet());

        match request.cookies().get_private("oauth_token") {
            Some(oauth_token) => {
                let json_account: Result<mastodon_api::Account, serde_json::Error> =
                    serde_json::from_value(
                        mastodon_api::controller::account_by_oauth_token(
                            &PooledConnection(database::POOL.get().unwrap()),
                            oauth_token.value().to_string(),
                        )
                        .into(),
                    );
                match json_account {
                    Ok(mastodon_api_account) => {
                        account = Some(mastodon_api_account);
                        token = Some(oauth_token.value().to_string());
                    }
                    Err(_) => (),
                }
            }
            None => (),
        }

        match &account {
            Some(local_account) => {
                context.insert(String::from("authenticated_account"), true.to_string());
                context.insert(
                    String::from("authenticated_account_display_name"),
                    (*local_account.display_name).to_string(),
                );
                context.insert(
                    String::from("authenticated_account_avatar"),
                    (*local_account.avatar).to_string(),
                );
                context.insert(
                    String::from("authenticated_account_id"),
                    (*local_account.id).to_string(),
                );
            }
            None => {
                context.insert(String::from("authenticated_account"), false.to_string());
                context.insert(
                    String::from("authenticated_account_display_name"),
                    String::from("Guest"),
                );
                context.insert(
                    String::from("authenticated_account_avatar"),
                    String::from("/static/assets/default_avatar.png"),
                );
                context.insert(String::from("authenticated_account_id"), String::from(""));
            }
        }

        context.extend(renderer::context_notifications(
            &PooledConnection(database::POOL.get().unwrap()),
            &token,
        ));

        return Outcome::Success(Configuration {
            account: account,
            context: context,
            token: token,
        });
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
