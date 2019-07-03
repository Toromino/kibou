pub mod controller;
pub mod routes;

use activity;
use actor;
use database;
use env;
use rocket::request;
use rocket::request::FromRequest;
use rocket::request::Request;
use rocket::Outcome;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use serde_json;

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
pub struct AuthorizationHeader(pub String);

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
    pub urls: serde_json::Value,
    pub stats: serde_json::Value,
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

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub created_at: String,
    pub account: Account,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
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

#[derive(FromForm)]
pub struct RegistrationForm {
    // Properties acctording to
    // - https://docs.joinmastodon.org/api/rest/accounts/#post-api-v1-accounts
    pub username: String,
    pub email: String,
    pub password: String,
    // Optional values in Kibou, as they're not used by the backend (yet?)
    pub agreement: Option<String>,
    pub locale: Option<String>,
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
    let database = database::establish_connection();
    json!(Instance {
        uri: format!(
            "{base_scheme}://{base_domain}",
            base_scheme = env::get_value(String::from("endpoint.base_scheme")),
            base_domain = env::get_value(String::from("endpoint.base_domain"))
        ),
        title: env::get_value(String::from("node.name")),
        description: env::get_value(String::from("node.description")),
        email: env::get_value(String::from("node.contact_email")),
        version: String::from("2.3.0 (compatible; Kibou 0.1)"),
        thumbnail: None,
        // Kibou does not support Streaming_API yet, but this value is not nullable according to
        // Mastodon-API's specifications, so that is why it is showing an empty value instead
        urls: serde_json::json!({"streaming_api": ""}),
        // `domain_count` always stays 0 as Kibou does not keep data about remote nodes
        stats: serde_json::json!({"user_count": actor::count_local_actors(&database).unwrap_or_else(|_| 0),
        "status_count": activity::count_local_ap_notes(&database).unwrap_or_else(|_| 0),
        "domain_count": 0}),
        languages: vec![],
        contact_account: None
    })
}

pub fn parse_authorization_header(header: &str) -> String {
    let header_vec: Vec<&str> = header.split(" ").collect();

    return header_vec[1].replace("\")", "").to_string();
}
