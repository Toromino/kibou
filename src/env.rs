extern crate config;

pub fn get_value(key: String) -> String
{
    let mut config = config::Config::default();

    // TODO: Find config file based on ROCKET_ENV
    config.merge(config::File::with_name("env.development.toml"));

    if config.get_str(&key).is_ok() { config.get_str(&key).ok().unwrap() }
    else
    {
        eprintln!("Key '{}' in environment config not found:", &key);
        String::from("")
    }
}
