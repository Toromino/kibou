#![feature(proc_macro_hygiene, decl_macro)]
#![feature(custom_attribute)]

extern crate base64;
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
mod actor;
mod database;
mod env;
mod kibou_api;
mod tests;
mod web_handler;
mod well_known;

fn main()
{
   // TODO: Determine the environment Rocket is running in (ROCKET_ENV)
   // We are currently just assuming a development enviroment

    let mut rocket_config = rocket::config::Config::build(rocket::config::Environment::Development)
    .address(env::get_value("endpoint.host".to_string()))
    .port(env::get_value("endpoint.port".to_string()).parse::<u16>().unwrap());

    // Launching Rocket with our own environment config
    rocket_app(rocket_config.unwrap()).launch();
}

fn rocket_app(config: rocket::config::Config) -> rocket::Rocket
{
    rocket::custom(config)

    .mount("/", routes![activitypub::routes::actor, activitypub::routes::actor_inbox, activitypub::routes::object, activitypub::routes::inbox])
    .mount("/", routes![well_known::webfinger::webfinger])
}
