extern crate kibou;

use kibou::env;
use kibou::rocket_app;

fn main() {
    // TODO: Determine the environment Rocket is running in (ROCKET_ENV)
    // We are currently just assuming a development enviroment

    let rocket_config = rocket::config::Config::build(rocket::config::Environment::Development)
        .address(env::get_value("endpoint.host".to_string()))
        .port(
            env::get_value("endpoint.port".to_string())
                .parse::<u16>()
                .unwrap(),
        );

    // Launching Rocket with our own environment config
    rocket_app(rocket_config.unwrap()).launch();
}
