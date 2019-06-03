extern crate config;

pub fn get_value(key: String) -> String {
    let mut config = config::Config::default();

    set_default_config_values(&mut config);

    // TODO: Find config file based on ROCKET_ENV
    config
        .merge(config::File::with_name("env.development.toml"))
        .expect("Environment config not found!");

    if config.get_str(&key).is_ok() {
        config.get_str(&key).ok().unwrap()
    } else {
        eprintln!("Key '{}' in environment config not found", &key);
        String::from("")
    }
}

fn set_default_config_values(config: &mut config::Config) {
    // Serve nodeinfo by default, but provide admins with a way
    // to disable it in the config file.
    config.set_default("nodeinfo.enabled", true).unwrap();
}
