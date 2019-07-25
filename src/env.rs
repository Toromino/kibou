use rocket::config::Environment;

pub fn get_value(key: String) -> String {
    let mut config = config::Config::default();
    let environment = match Environment::active() {
        Ok(Environment::Development) => "development",
        Ok(Environment::Staging) => "staging",
        Ok(Environment::Production) => "production",
        Err(_) => "development",
    };

    set_default_config_values(&mut config);

    config
        .merge(config::File::with_name(&format!(
            "env.{}.toml",
            environment
        )))
        .expect("Environment config not found!");

    match config.get_str(&key) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Key '{}' not found in config", &key);
            return String::from("");
        }
    }
}

fn set_default_config_values(config: &mut config::Config) {
    // Serve nodeinfo by default, but provide admins with a way
    // to disable it in the config file.
    config.set_default("nodeinfo.enabled", true).unwrap();
}
