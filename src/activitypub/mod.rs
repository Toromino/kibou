pub mod activity;
pub mod actor;
pub mod controller;
pub mod routes;
pub mod validator;

use rocket::http::Status;
use rocket::request;
use rocket::request::FromRequest;
use rocket::request::Request;
use rocket::Outcome;
use web_handler::http_signatures::HTTPSignature;

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
