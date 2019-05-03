#![feature(proc_macro_hygiene, decl_macro)]

extern crate getopts;
extern crate kibou;
extern crate rocket;
extern crate rocket_contrib;

use getopts::Options;
use rocket_contrib::templates::Template;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut options = Options::new();
    options.reqopt("u", "url", "instance url", "https://instance.tld");

    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string()),
    };

    unsafe {
        kibou::raito_fe::BYPASS_API = &false;
        kibou::raito_fe::MASTODON_API_BASE_URI =
            Box::leak(Box::new(matches.opt_str("url").unwrap()));
    }
    rocket::ignite()
        .mount("/", kibou::raito_fe::get_routes())
        .attach(Template::fairing())
        .launch();
}
