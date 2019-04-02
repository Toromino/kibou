use actor;
use database;
use mastodon_api::status::Emoji;
use oauth::token::verify_token;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Account {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#account
    pub id: String,
    pub username: String,
    pub acct: String,
    pub display_name: String,
    pub locked: bool,
    pub created_at: String,
    pub followers_count: i64,
    pub following_count: i64,
    pub statuses_count: i64,
    pub note: String,
    pub url: String,
    pub avatar: String,
    pub avatar_static: String,
    pub header: String,
    pub header_static: String,
    pub emojis: Vec<Emoji>,
    pub source: Option<Source>,
}
#[derive(Serialize, Deserialize)]
pub struct Source {
    pub privacy: Option<String>,
    pub sensitive: Option<bool>,
    pub language: Option<String>,
    pub note: String,
    pub fields: Vec<String>,
}

pub fn get_json_by_id(id: i64) -> JsonValue {
    let database = database::establish_connection();

    match actor::get_actor_by_id(&database, id) {
        Ok(actor) => json!(serialize(actor, false)),
        Err(_) => json!({"error": "User not found."}),
    }
}

pub fn serialize(mut actor: actor::Actor, include_source: bool) -> Account {
    let mut new_account = Account {
        id: actor.id.to_string(),
        username: actor.preferred_username.clone(),
        acct: actor.get_acct(),
        display_name: actor.username.unwrap_or_else(|| String::from("")),
        locked: false,
        created_at: actor.created.to_string(),
        followers_count: 0,
        following_count: 0,
        statuses_count: 0,
        note: actor.summary.unwrap_or_else(|| String::from("")),
        url: actor.actor_uri,
        avatar: actor.icon.clone().unwrap_or_else(|| String::from("")),
        avatar_static: actor.icon.unwrap_or_else(|| String::from("")),
        header: String::from(""),
        header_static: String::from(""),
        emojis: vec![],
        source: None,
    };

    if include_source {
        new_account.source = Some(Source {
            privacy: None,
            sensitive: None,
            language: None,
            note: new_account.note.clone(),
            fields: vec![],
        });
    }

    return new_account;
}

pub fn get_json_by_oauth_token(token: String) -> JsonValue {
    let database = database::establish_connection();

    match verify_token(&database, token) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, token.actor) {
            Ok(actor) => json!(serialize(actor, true)),
            Err(_) => json!({"error": "No user is associated to this token!"}),
        },
        Err(_) => json!({"error": "Token invalid!"}),
    }
}
