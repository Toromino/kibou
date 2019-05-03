pub mod account;
pub mod api_controller;
pub mod routes;
pub mod status;
pub mod timeline;

use std::fs::File;
use std::io::prelude::*;

pub static mut BYPASS_API: &'static bool = &false;
pub static mut MASTODON_API_BASE_URI: &'static str = "127.0.0.1";

pub fn get_routes() -> Vec<rocket::Route> {
    routes![
        routes::about,
        routes::account,
        routes::actor,
        routes::global_timeline,
        routes::index,
        routes::object,
        routes::public_timeline,
        routes::view_status
    ]
}

pub fn get_stylesheet() -> String {
    let mut style = File::open("static/raito_fe/themes/raito_light.css").expect("theme not found");

    let mut contents = String::new();
    style
        .read_to_string(&mut contents)
        .expect("something went wrong reading the file");
    return contents;
}
