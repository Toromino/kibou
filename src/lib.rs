#![feature(proc_macro_hygiene, decl_macro)]
#![feature(custom_attribute)]

extern crate base64;
extern crate bcrypt;
extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate openssl;
extern crate pem;
extern crate regex;
extern crate reqwest;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate uuid;

mod activity;
mod activitypub;
pub mod actor;
pub mod database;
pub mod env;
mod kibou_api;
mod mastodon_api;
mod oauth;
mod tests;
mod timeline;
mod web_handler;
mod well_known;

pub fn rocket_app(config: rocket::config::Config) -> rocket::Rocket {
    rocket::custom(config)
        .mount(
            "/",
            routes![
                activitypub::routes::activity,
                activitypub::routes::actor,
                activitypub::routes::actor_inbox,
                activitypub::routes::object,
                activitypub::routes::inbox
            ],
        )
        .mount(
            "/",
            routes![
                mastodon_api::routes::account,
                mastodon_api::routes::account_verify_credentials,
                mastodon_api::routes::application,
                mastodon_api::routes::home_timeline,
                mastodon_api::routes::instance,
                mastodon_api::routes::status,
                mastodon_api::routes::status_post,
                mastodon_api::routes::public_timeline,
                mastodon_api::routes::options_account,
                mastodon_api::routes::options_account_verify_credentials,
                mastodon_api::routes::options_home_timeline,
                mastodon_api::routes::options_instance,
                mastodon_api::routes::options_public_timeline,
                mastodon_api::routes::options_status
            ],
        )
        .mount(
            "/",
            routes![
                oauth::routes::authorize,
                oauth::routes::authorize_result,
                oauth::routes::token
            ],
        )
        .mount(
            "/",
            routes![
                well_known::nodeinfo::nodeinfo,
                well_known::nodeinfo::nodeinfo_v2,
                well_known::nodeinfo::nodeinfo_v2_1,
                well_known::webfinger::webfinger
            ],
        )
        .attach(rocket_contrib::templates::Template::fairing())
}
