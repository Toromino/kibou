extern crate diesel;
extern crate getopts;
extern crate kibou;

use getopts::Options;
use kibou::actor;
use kibou::database;
use kibou::env;
use std::env::args;

fn main() {
    let database = database::establish_connection();
    let args: Vec<String> = std::env::args().collect();

    let mut options = Options::new();
    options.reqopt(
        "e",
        "email",
        "e-mail address for the new user",
        "alyssa@example.com",
    );
    options.reqopt(
        "p",
        "password",
        "password for the new user",
        "MySecretPassword",
    );
    options.reqopt("u", "username", "username for the new user", "alyssatest");
    options.optopt(
        "s",
        "summary",
        "a summary (bio) for the new user",
        "I am a Kibou Test-Actor, how are you doing?",
    );

    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string()),
    };

    let mut new_actor = actor::Actor {
        id: 0,
        email: Some(matches.opt_str("email").unwrap()),
        password: Some(matches.opt_str("password").unwrap()),
        actor_uri: format!(
            "{base_scheme}://{base_domain}/actors/{username}",
            base_scheme = env::get_value(String::from("endpoint.base_scheme")),
            base_domain = env::get_value(String::from("endpoint.base_domain")),
            username = matches.opt_str("username").unwrap()
        ),
        username: Some(matches.opt_str("username").unwrap()),
        preferred_username: matches.opt_str("username").unwrap(),
        summary: matches.opt_str("summary"),
        followers: serde_json::json!({"activitypub": []}),
        inbox: None,
        icon: None,
        local: true,
        keys: serde_json::json!({}),
    };

    actor::create_actor(&database, &mut new_actor)
}
