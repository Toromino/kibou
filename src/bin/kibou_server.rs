extern crate kibou;

use kibou::env;
use kibou::rocket_app;

fn main() {
    let rocket_config = rocket::config::Config::build(
        rocket::config::Environment::active()
            .expect("Unknown ROCKET_ENV value! (enum: {Development, Staging, Production})"),
    )
    .address(env::get_value("endpoint.host".to_string()))
    .log_level(rocket::config::LoggingLevel::Normal)
    .port(
        env::get_value("endpoint.port".to_string())
            .parse::<u16>()
            .unwrap(),
    )
    .workers(
        env::get_value("endpoint.workers".to_string())
            .parse::<u16>()
            .unwrap_or_else(|_| 2),
    );

    unsafe {
        kibou::raito_fe::BYPASS_API = &true;
    }

    // Launching Rocket with our own environment config
    rocket_app(rocket_config.unwrap()).launch();
}
