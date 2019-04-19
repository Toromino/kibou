pub mod activity;
pub mod actor;
pub mod controller;
pub mod routes;
pub mod validator;

use rocket::http::ContentType;
use rocket::http::MediaType;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::response::{self, Responder, Response};
use rocket::Outcome;
use std::io::Cursor;
use web_handler::http_signatures::HTTPSignature;

pub struct ActivitypubMediatype(bool);
pub struct ActivitystreamsResponse(String);

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

impl<'a, 'r> FromRequest<'a, 'r> for HTTPSignature {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<HTTPSignature, ()> {
        let content_length_vec: Vec<_> = request.headers().get("Content-Length").collect();
        let date_vec: Vec<_> = request.headers().get("Date").collect();
        let digest_vec: Vec<_> = request.headers().get("Digest").collect();
        let host_vec: Vec<_> = request.headers().get("Host").collect();
        let signature_vec: Vec<_> = request.headers().get("Signature").collect();

        if signature_vec.is_empty() {
            return Outcome::Failure((Status::BadRequest, ()));
        } else {
            return Outcome::Success(HTTPSignature {
                content_length: content_length_vec.get(0).unwrap_or_else(|| &"").to_string(),
                date: date_vec.get(0).unwrap_or_else(|| &"").to_string(),
                digest: digest_vec.get(0).unwrap_or_else(|| &"").to_string(),
                endpoint: format!("post {}", request.route().unwrap().to_string()),
                host: host_vec.get(0).unwrap_or_else(|| &"").to_string(),
                signature: signature_vec[0].to_string(),
            });
        }
    }
}
