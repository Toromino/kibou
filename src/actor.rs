use bcrypt;
use chrono::NaiveDateTime;
use database::models::QueryActor;
use database::schema::actors;
use database::schema::actors::dsl::*;
use diesel::pg::PgConnection;
use diesel::query_dsl::QueryDsl;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_query;
use diesel::ExpressionMethods;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::sign::Signer;
use pem::Pem;
use serde_json;
use url::Url;

pub struct Actor {
    pub id: i64,
    pub email: Option<String>,
    pub password: Option<String>,
    pub actor_uri: String,
    pub username: Option<String>,
    pub preferred_username: String,
    pub summary: Option<String>,
    pub followers: serde_json::Value,
    pub inbox: Option<String>,
    pub icon: Option<String>,
    pub local: bool,
    pub keys: serde_json::Value,
    pub created: NaiveDateTime,
}

impl Actor {
    /// Generates a new keypair and returns it as a serde_json::Value
    ///
    /// # Tests
    ///
    /// Tests for this function are in `tests/actor.rs`
    /// - generate_new_keys()
    fn generate_new_keys(&mut self) -> serde_json::Value {
        let rsa_keys = Rsa::generate(2048).unwrap();
        let public_key = Pem {
            tag: String::from("PUBLIC KEY"),
            contents: rsa_keys.public_key_to_der().unwrap(),
        };

        let private_key = Pem {
            tag: String::from("PRIVATE KEY"),
            contents: rsa_keys.private_key_to_der().unwrap(),
        };

        serde_json::json!({
            "public": pem::encode(&public_key),
            "private": pem::encode(&private_key)
        })
    }

    pub fn get_acct(&mut self) -> String {
        if self.local {
            self.preferred_username.to_string()
        } else {
            let url = Url::parse(&self.actor_uri).unwrap();
            format!(
                "{username}@{host}",
                username = self.preferred_username,
                host = url.host_str().unwrap()
            )
        }
    }

    pub fn get_public_key(&mut self) -> String {
        let parsed_public_key = self.keys["public"].as_str();
        parsed_public_key.unwrap().to_string()
    }

    pub fn get_private_key(&mut self) -> String {
        let parsed_private_key = self.keys["private"].as_str();
        parsed_private_key.unwrap().to_string()
    }

    /// Signs a string with the actor's private key and returns the newly signed string encoded in
    /// base64
    ///
    /// # Parameters
    ///
    /// * `request_string` - String | The string we want to get signed
    ///
    /// # Tests
    ///
    /// Tests for this function are in `tests/actor.rs`
    /// - sign()
    pub fn sign(&mut self, request_string: String) -> String {
        let private_key = self.get_private_key();
        let pem_decoded = pem::parse(private_key).unwrap();
        let pkey =
            PKey::from_rsa(openssl::rsa::Rsa::private_key_from_der(&pem_decoded.contents).unwrap())
                .unwrap();
        let mut signer = Signer::new(MessageDigest::sha256(), &pkey).unwrap();

        signer.update(&request_string.into_bytes()).unwrap();
        base64::encode(&signer.sign_to_vec().unwrap())
    }

    /// Updates the keypair of a local actor
    ///
    /// # Tests
    ///
    /// Tests for this function are in `tests/actor.rs`
    /// - update_local_keys()
    pub fn update_local_keys(&mut self) {
        self.keys = self.generate_new_keys();
    }
}

fn serialize_actor(sql_actor: QueryActor) -> Actor {
    Actor {
        id: sql_actor.id,
        email: sql_actor.email,
        password: sql_actor.password,
        actor_uri: sql_actor.actor_uri,
        username: sql_actor.username,
        preferred_username: sql_actor.preferred_username,
        summary: sql_actor.summary,
        inbox: sql_actor.inbox,
        icon: sql_actor.icon,
        keys: sql_actor.keys,
        local: sql_actor.local,
        followers: sql_actor.followers,
        created: sql_actor.created,
    }
}

pub fn authorize(
    db_connection: &PgConnection,
    _preferred_username: &str,
    _password: String,
) -> Result<bool, diesel::result::Error> {
    match actors
        .filter(preferred_username.eq(_preferred_username))
        .limit(1)
        .first::<QueryActor>(db_connection)
    {
        Ok(actor) => Ok(
            bcrypt::verify(_password.into_bytes(), &actor.password.unwrap())
                .unwrap_or_else(|_| false),
        ),
        Err(e) => Err(e),
    }
}

pub fn is_actor_followed_by(
    db_connection: &PgConnection,
    actor: &Actor,
    followee: &str,
) -> Result<bool, diesel::result::Error> {
    match actors
        .filter(actor_uri.eq(&actor.actor_uri))
        .limit(1)
        .first::<QueryActor>(db_connection)
    {
        Ok(actor) => match actor.followers["activitypub"].as_array() {
            Some(follows) => {
                let mut follow_exists: bool = false;

                for follow in follows {
                    if follow["href"].as_str() == Some(followee) {
                        follow_exists = true;
                    }
                }

                Ok(follow_exists)
            }
            None => Ok(false),
        },
        Err(e) => Err(e),
    }
}

pub fn get_actor_by_acct(
    db_connection: &PgConnection,
    acct: String,
) -> Result<Actor, diesel::result::Error> {
    if acct.contains("@") {
        let acct_split = acct.split('@');
        let acct_vec = acct_split.collect::<Vec<&str>>();

        match sql_query(format!("SELECT * FROM actors WHERE preferred_username = '{username}' AND actor_uri LIKE '%{uri}/%' LIMIT 1;", username=acct_vec[0], uri=acct_vec[1]))
             .clone()
             .load::<QueryActor>(db_connection)
         {
             Ok(actor) => {
                 if !actor.is_empty()
                 {
                     let new_actor = std::borrow::ToOwned::to_owned(&actor[0]);
                     Ok(serialize_actor(new_actor))
                 } else { Err(diesel::result::Error::NotFound) }
             },
             Err(e) => Err(e),
         }
    } else {
        get_local_actor_by_preferred_username(&db_connection, acct)
    }
}

pub fn get_actor_by_id(
    db_connection: &PgConnection,
    _id: i64,
) -> Result<Actor, diesel::result::Error> {
    match actors
        .filter(id.eq(_id))
        .limit(1)
        .first::<QueryActor>(db_connection)
    {
        Ok(actor) => Ok(serialize_actor(actor)),
        Err(e) => Err(e),
    }
}

pub fn get_actor_by_uri(
    db_connection: &PgConnection,
    _actor_uri: &str,
) -> Result<Actor, diesel::result::Error> {
    match actors
        .filter(actor_uri.eq(_actor_uri))
        .limit(1)
        .first::<QueryActor>(db_connection)
    {
        Ok(actor) => Ok(serialize_actor(actor)),
        Err(e) => Err(e),
    }
}

pub fn get_actor_followees(
    db_connection: &PgConnection,
    _actor_uri: &str,
) -> Result<Vec<String>, diesel::result::Error> {
    match sql_query(format!(
        "WITH actor \
        AS ( SELECT id, email, password, actor_uri, username, preferred_username, summary, inbox, icon, keys, created, modified, local, jsonb_array_elements(followers->'activitypub') \
        AS followers FROM actors) \
        SELECT * FROM actor \
        WHERE (followers->>'href') = '{uri}';",
        uri = _actor_uri
    ))
        .load::<QueryActor>(db_connection)
        {
            Ok(actor_vec) => {
                let mut followings: Vec<String> = vec![];

                for actor in actor_vec {
                    followings.push(actor.actor_uri);
                }

                return Ok(followings);
            },
            Err(e) => Err(e),
        }
}

/// Runs a database query based on a local actor's preferred_username, returns either
/// an actor::Actor or a diesel::result::Error
///
/// # Parameters
///
/// * `db_connection` - &PgConnection | Reference to a database connection
/// * `preferred_username` - String | The preferred_username that is being queried
///
/// # Tests
///
/// Tests for this function are in `tests/actor.rs`
/// - get_local_actor_by_preferred_username()
pub fn get_local_actor_by_preferred_username(
    db_connection: &PgConnection,
    _preferred_username: String,
) -> Result<Actor, diesel::result::Error> {
    match actors
        .filter(preferred_username.eq(_preferred_username))
        .filter(local.eq(true))
        .limit(1)
        .first::<QueryActor>(db_connection)
    {
        Ok(actor) => Ok(serialize_actor(actor)),
        Err(e) => Err(e),
    }
}

pub fn count_local_actors(db_connection: &PgConnection) -> Result<usize, diesel::result::Error> {
    match actors
        .filter(local.eq(true))
        .load::<QueryActor>(db_connection)
    {
        Ok(actor_arr) => Ok(actor_arr.len()),
        Err(e) => Err(e),
    }
}

/// Creates a new actor
///
/// # Parameters
///
/// * `db_connection` - &PgConnection | Reference to a database connection
/// * `actor` - actor::Actor | An actor serialized in an actor::Actor struct
///
/// # Tests
///
/// Tests for this function are in `tests/actor.rs`
/// - create_local_actor()
/// - create_remote_actor()
/// - create_actor_with_optional_values()
pub fn create_actor(db_connection: &PgConnection, actor: &mut Actor) {
    if actor.local && actor.keys == serde_json::json!({}) {
        actor.update_local_keys();
    }

    if actor.local {
        actor.password = Some(
            bcrypt::hash(
                actor.password.to_owned().unwrap().into_bytes(),
                bcrypt::DEFAULT_COST,
            )
            .unwrap(),
        );
    }

    let new_actor = (
        email.eq(&actor.email),
        password.eq(&actor.password),
        actor_uri.eq(&actor.actor_uri),
        username.eq(&actor.username),
        preferred_username.eq(&actor.preferred_username),
        summary.eq(&actor.summary),
        inbox.eq(&actor.inbox),
        icon.eq(&actor.icon),
        local.eq(&actor.local),
        keys.eq(&actor.keys),
    );

    diesel::insert_into(actors::table)
        .values(new_actor)
        .execute(db_connection)
        .expect("Error creating user");
}

/// Deletes an actor base on their actor_uri
///
/// # Parameters
///
/// * `db_connection` - &PgConnection | Reference to a database connection
/// * `actor` - actor::Actor | An actor serialized in an actor::Actor struct
///
/// # Tests
///
/// Tests for this function are at `tests/actor.rs`
/// - delete_local_actor()
/// - delete_remote_actor()
pub fn delete(db_connection: &PgConnection, actor: &mut Actor) {
    diesel::delete(actors.filter(actor_uri.eq(&actor.actor_uri)))
        .execute(db_connection)
        .expect("Error deleting user");
}

pub fn update_followers(db_connection: &PgConnection, actor: &mut Actor) {
    diesel::update(actors.filter(actor_uri.eq(&actor.actor_uri)))
        .set(followers.eq(&actor.followers))
        .execute(db_connection)
        .expect("Error updating followers");
}
