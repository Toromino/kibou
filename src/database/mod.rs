pub mod models;
pub mod schema;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use env;
use regex::Regex;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub struct PooledConnection(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

lazy_static! {
    pub static ref POOL: Pool = initialize_pool();
}

impl<'a, 'r> FromRequest<'a, 'r> for PooledConnection {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<PooledConnection, Self::Error> {
        match POOL.get() {
            Ok(db_connection) => Outcome::Success(PooledConnection(db_connection)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl std::ops::Deref for PooledConnection {
    type Target = PgConnection;
    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

#[deprecated]
pub fn establish_connection() -> PooledConnection {
    // Originally this always established a new database connection, now this is just gonna
    // return a new connection from the pool until this function can be removed completely.
    //
    // TODO: Remove all references of this function!

    return PooledConnection(POOL.get().unwrap());
}

pub fn initialize_pool() -> Pool {
    let connection_manager = ConnectionManager::<PgConnection>::new(prepare_postgres_url());

    // TODO: Eventually replace this with r2d2::Pool::builder()
    return Pool::new(connection_manager).expect("Could not initialize database pool!");
}

pub fn runtime_escape(value: &str) -> String {
    let escape_regex = Regex::new(r"[a-zA-Z0-9_:\\.\-\\/]").unwrap();
    value
        .chars()
        .filter(|&c| escape_regex.is_match(&c.to_string()))
        .collect()
}

fn prepare_postgres_url() -> String {
    return format!(
        "postgres://{username}:{password}@{host}/{database}",
        username = env::get_value("database.username".to_string()),
        password = env::get_value("database.password".to_string()),
        host = env::get_value("database.hostname".to_string()),
        database = env::get_value("database.database".to_string())
    );
}
