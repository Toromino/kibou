pub mod account;
pub mod application;
pub mod routes;
pub mod status;
pub mod timeline;

use env;
use mastodon_api::account::Account;
use rocket::http::Status;
use rocket::request;
use rocket::request::FromRequest;
use rocket::request::Request;
use rocket::Outcome;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AuthorizationHeader(String);

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
            return Outcome::Failure((Status::BadRequest, ()));
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
        version: String::from("2.3.0"),
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
