pub mod activity;
pub mod actor;
pub mod controller;
pub mod routes;
pub mod validator;

use base64;
use rocket::http::ContentType;
use rocket::http::MediaType;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::response::{self, Responder, Response};
use rocket::Outcome;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use web::http_signatures::Signature;

pub struct ActivitypubMediatype(bool);
pub struct ActivitystreamsResponse(String);

// ActivityStreams2/AcitivityPub properties are expressed in CamelCase
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub _type: String,
    pub content: Option<String>,
    pub url: String,
    pub name: Option<String>,
    pub mediaType: Option<String>,
}

impl<'a, 'r> FromRequest<'a, 'r> for ActivitypubMediatype {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<ActivitypubMediatype, ()> {
        let activitypub_default = MediaType::with_params(
            "application",
            "ld+json",
            ("profile", "https://www.w3.org/ns/activitystreams"),
        );
        let activitypub_lite = MediaType::new("application", "activity+json");

        match request.accept() {
            Some(accept) => {
                if accept
                    .media_types()
                    .find(|t| t == &&activitypub_default)
                    .is_some()
                    || accept
                        .media_types()
                        .find(|t| t == &&activitypub_lite)
                        .is_some()
                {
                    Outcome::Success(ActivitypubMediatype(true))
                } else {
                    Outcome::Forward(())
                }
            }
            None => Outcome::Forward(()),
        }
    }
}

impl<'r> Responder<'r> for ActivitystreamsResponse {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        Response::build()
            .header(ContentType::new("application", "activity+json"))
            .sized_body(Cursor::new(self.0))
            .ok()
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Signature {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Signature, ()> {
        let content_length_vec: Vec<_> = request.headers().get("Content-Length").collect();
        let content_type_vec: Vec<_> = request.headers().get("Content-Type").collect();
        let date_vec: Vec<_> = request.headers().get("Date").collect();
        let digest_vec: Vec<_> = request.headers().get("Digest").collect();
        let host_vec: Vec<_> = request.headers().get("Host").collect();
        let signature_vec: Vec<_> = request.headers().get("Signature").collect();

        if signature_vec.is_empty() {
            return Outcome::Failure((Status::BadRequest, ()));
        } else {
            let parsed_signature: HashMap<String, String> = signature_vec[0]
                .replace("\"", "")
                .to_string()
                .split(',')
                .map(|kv| kv.split('='))
                .map(|mut kv| (kv.next().unwrap().into(), kv.next().unwrap().into()))
                .collect();

            let headers: Vec<&str> = parsed_signature["headers"].split_whitespace().collect();
            let route = request.uri().to_string();

            return Outcome::Success(Signature {
                algorithm: None,
                content_length: Some(content_length_vec.get(0).unwrap_or_else(|| &"").to_string()),
                content_type: Some(content_type_vec.get(0).unwrap_or_else(|| &"").to_string()),
                date: date_vec.get(0).unwrap_or_else(|| &"").to_string(),
                digest: Some(digest_vec.get(0).unwrap_or_else(|| &"").to_string()),
                headers: headers.iter().map(|header| header.to_string()).collect(),
                host: host_vec.get(0).unwrap_or_else(|| &"").to_string(),
                key_id: None,
                request_target: Some(route),
                signature: String::new(),
                signature_in_bytes: Some(
                    base64::decode(&parsed_signature["signature"].to_owned().into_bytes()).unwrap(),
                ),
            });
        }
    }
}
