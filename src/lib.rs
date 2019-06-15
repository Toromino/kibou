#![feature(proc_macro_hygiene, decl_macro)]
#![feature(custom_attribute)]

extern crate base64;
extern crate bcrypt;
#[macro_use]
extern crate cached;
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
mod html;
mod kibou_api;
mod mastodon_api;
mod oauth;
pub mod raito_fe;
mod tests;
mod timeline;
mod web_handler;
mod well_known;

pub fn rocket_app(config: rocket::config::Config) -> rocket::Rocket {
    let mut app: rocket::Rocket = rocket::custom(config)
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
        .mount("/", routes![kibou_api::routes::activities])
        .mount(
            "/",
            routes![
                mastodon_api::routes::account,
                mastodon_api::routes::account_follow,
                mastodon_api::routes::account_statuses,
                mastodon_api::routes::account_unfollow,
                mastodon_api::routes::account_verify_credentials,
                mastodon_api::routes::application,
                mastodon_api::routes::home_timeline,
                mastodon_api::routes::instance,
                mastodon_api::routes::status,
                mastodon_api::routes::status_context,
                mastodon_api::routes::status_post,
                mastodon_api::routes::public_timeline,
                mastodon_api::routes::options_account,
                mastodon_api::routes::options_account_statuses,
                mastodon_api::routes::options_account_verify_credentials,
                mastodon_api::routes::options_home_timeline,
                mastodon_api::routes::options_instance,
                mastodon_api::routes::options_public_timeline,
                mastodon_api::routes::options_status
            ],
        )
        .mount("/", raito_fe::get_routes())
        .mount(
            "/",
            routes![
                oauth::routes::authorize,
                oauth::routes::authorize_result,
                oauth::routes::token
            ],
        )
        .mount("/", routes![well_known::webfinger::webfinger])
        .mount(
            "/static",
            rocket_contrib::serve::StaticFiles::from("static"),
        )
        .attach(rocket_contrib::templates::Template::fairing());

    // Avoid mounting nodeinfo routes if the admin has disabled
    // nodeinfo in the config file.
    if should_mount_nodeinfo() {
        app = app.mount("/", well_known::nodeinfo::get_routes());
    }

    app
}

// Returns whether nodeinfo routes should be mounted based
// on the value of a configuration entry.
fn should_mount_nodeinfo() -> bool {
    let key = String::from("nodeinfo.enabled");
    let value: String = env::get_value(key).to_lowercase();

    // Nodeinfo will only be disabled if its configuration value is
    // explicitly set to false, "false", or any uppercase equivalent.
    value.as_str() != "false"
}
