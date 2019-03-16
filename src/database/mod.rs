pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use env;

pub fn establish_connection() -> PgConnection {
    let database_url = format!(
        "postgres://{username}:{password}@{host}/{database}",
        username = env::get_value("database.username".to_string()),
        password = env::get_value("database.password".to_string()),
        host = env::get_value("database.hostname".to_string()),
        database = env::get_value("database.database".to_string())
    );

    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!(format!("Could not connect to {url}", url = &database_url)))
}
