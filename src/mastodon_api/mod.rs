pub mod controller;
pub mod routes;

use env;
use rocket::request;
use rocket::request::FromRequest;
use rocket::request::Request;
use rocket::Outcome;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::HashMap;
use std::sync::Mutex;

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

#[derive(FromForm)]
pub struct ApplicationForm {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#application
    // - https://docs.joinmastodon.org/api/rest/apps/#post-api-v1-apps
    pub client_name: String,
    pub redirect_uris: String,
    pub scopes: String,
    pub website: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub url: String,
    pub remote_url: Option<String>,
    pub preview_url: String,
    pub text_url: Option<String>,
    pub meta: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct AuthorizationHeader(String);

#[derive(Serialize, Deserialize)]
pub struct Emoji {
    pub shortcode: String,
    pub static_url: String,
    pub url: String,
    pub visible_in_picker: bool,
}

#[derive(FromForm)]
pub struct HomeTimeline {
    pub max_id: Option<i64>,
    pub since_id: Option<i64>,
    pub min_id: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct Instance {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#instance
    pub uri: String,
    pub title: String,
    pub description: String,
    pub email: String,
    pub version: String,
    pub thumbnail: Option<String>,
    pub urls: String,
    pub stats: String,
    pub languages: Vec<String>,
    pub contact_account: Option<Account>,
}

#[derive(Serialize, Deserialize)]
pub struct Mention {
    pub url: String,
    pub username: String,
    pub acct: String,
    pub id: String,
}

#[derive(FromForm)]
pub struct PublicTimeline {
    pub local: Option<bool>,
    pub only_media: Option<bool>,
    pub max_id: Option<i64>,
    pub since_id: Option<i64>,
    pub min_id: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct Relationship {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#relationship
    pub id: String,
    pub following: bool,
    pub followed_by: bool,
    pub blocking: bool,
    pub muting: bool,
    pub muting_notifications: bool,
    pub requested: bool,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Source {
    pub privacy: Option<String>,
    pub sensitive: Option<bool>,
    pub language: Option<String>,
    pub note: String,
    pub fields: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Status {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#status
    pub id: String,
    pub uri: String,
    pub url: Option<String>,
    pub account: Account,
    pub in_reply_to_id: Option<String>,
    pub in_reply_to_account_id: Option<String>,
    pub reblog: Option<serde_json::Value>,
    pub content: String,
    pub created_at: String,
    pub emojis: Vec<Emoji>,
    pub replies_count: i64,
    pub reblogs_count: i64,
    pub favourites_count: i64,
    pub reblogged: Option<bool>,
    pub favourited: Option<bool>,
    pub muted: Option<bool>,
    pub sensitive: bool,
    pub spoiler_text: String,
    pub visibility: String,
    pub media_attachments: Vec<Attachment>,
    pub mentions: Vec<Mention>,
    pub tags: Vec<Tag>,
    pub application: serde_json::Value,
    pub language: Option<String>,
    pub pinned: Option<bool>,
}

#[derive(FromForm)]
pub struct StatusForm {
    pub status: Option<String>,
    pub in_reply_to_id: Option<String>,
    pub media_ids: Option<String>,
    pub sensitive: Option<bool>,
    pub spoiler_text: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub url: String,
    pub history: Option<serde_json::Value>,
}

impl ToString for AuthorizationHeader {
    fn to_string(&self) -> String {
        format!("{:?}", &self)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AuthorizationHeader {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<AuthorizationHeader, ()> {
        let headers: Vec<_> = request.headers().get("Authorization").collect();
        if headers.is_empty() {
            return Outcome::Failure((rocket::http::Status::BadRequest, ()));
        } else {
            return Outcome::Success(AuthorizationHeader(headers[0].to_string()));
        }
    }
}

pub fn get_instance_info() -> JsonValue {
    json!(Instance {
        uri: format!(
            "{base_scheme}://{base_domain}",
            base_scheme = env::get_value(String::from("endpoint.base_scheme")),
            base_domain = env::get_value(String::from("endpoint.base_domain"))
        ),
        title: env::get_value(String::from("node.name")),
        description: env::get_value(String::from("node.description")),
        email: String::from(""),
        version: String::from("2.3.0 (compatible; Kibou 0.1)"),
        thumbnail: None,
        urls: String::from(""),
        stats: String::from(""),
        languages: vec![],
        contact_account: None
    })
}

pub fn parse_authorization_header(header: &str) -> String {
    let header_vec: Vec<&str> = header.split(" ").collect();

    return header_vec[1].replace("\")", "").to_string();
}
