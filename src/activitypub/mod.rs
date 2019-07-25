pub mod controller;
pub mod routes;
pub mod validator;

use activity;
use base64;
use chrono::Utc;
use database;
use env;
use rocket::data::{self, Data, FromDataSimple};
use rocket::http::ContentType;
use rocket::http::MediaType;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::response::{self, Responder, Response};
use rocket::Outcome;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use uuid::Uuid;
use web::http_signatures::Signature;

pub struct ActivitypubMediatype(bool);
pub struct ActivitystreamsResponse(String);

// ActivityStreams2/ActivityPub properties are expressed in CamelCase
#[allow(non_snake_case)]
#[derive(Clone, Deserialize, Serialize)]
pub struct Activity {
    // Properties according to
    // - https://www.w3.org/TR/activitystreams-core/#activities
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub _type: String,
    pub id: String,
    pub actor: String,
    pub object: serde_json::Value,
    pub published: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
}

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

#[allow(non_snake_case)]
#[derive(Clone, Deserialize, Serialize)]
pub struct Object {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub _type: String,
    pub id: String,
    pub published: String,
    pub attributedTo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inReplyTo: Option<String>,
    pub summary: Option<String>,
    pub content: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub tag: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "type")]
    pub _type: String,
    pub href: String,
    pub name: String,
}


// ActivityStreams2/AcitivityPub properties are expressed in CamelCase
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Actor {
    // Properties according to
    // - https://www.w3.org/TR/activitypub/#actor-objects
    // - https://www.w3.org/TR/activitystreams-core/#actors
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub _type: String,
    pub id: String,
    pub summary: Option<String>,
    pub following: String,
    pub followers: String,
    pub inbox: String,
    pub outbox: String,
    pub preferredUsername: String,
    pub name: Option<String>,
    pub publicKey: serde_json::Value,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<serde_json::Value>,
    pub endpoints: Option<serde_json::Value>,
}

// ActivityStreams2/AcitivityPub properties are expressed in CamelCase
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Outbox {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub id: String,
    pub totalItems: i64,
    pub orderedItems: Vec<Activity>,
}

pub fn add_follow(account: &str, source: &str, activity_id: &str) {
    let database = database::establish_connection();
    let mut actor = actor::get_actor_by_uri(&database, &account).unwrap();
    let followers: serde_json::Value = actor.followers["activitypub"].clone();

    let follow_data = serde_json::from_value(followers);

    if follow_data.is_ok() {
        let mut follow_data: Vec<serde_json::Value> = follow_data.unwrap();
        let new_follow_data = serde_json::json!({"href" : source,
        "follow_date": Utc::now().to_rfc3339().to_string(),
        "activity_id": activity_id});
        follow_data.push(new_follow_data);

        actor.followers["activitypub"] = serde_json::to_value(follow_data).unwrap();
        actor::update_followers(&database, actor);
    }
}

pub fn remove_follow(account: &str, source: &str) {
    let database = database::establish_connection();
    let mut actor = actor::get_actor_by_uri(&database, &account).unwrap();
    let followers: serde_json::Value = actor.followers["activitypub"].clone();

    let follow_data = serde_json::from_value(followers);

    if follow_data.is_ok() {
        let mut follow_data: Vec<serde_json::Value> = follow_data.unwrap();
        let index = follow_data
            .iter()
            .position(|ref follow| follow["href"] == source)
            .unwrap();
        follow_data.remove(index);

        actor.followers["activitypub"] = serde_json::to_value(follow_data).unwrap();
        actor::update_followers(&database, actor);
    }
}

// Refetches remote actor and detects changes to icon, username, keys and summary
pub fn refresh(uri: String) {
    std::thread::spawn(move || {
        let expiration_time: chrono::DateTime<Utc> = Utc::now() - Duration::days(2);
        let database = database::establish_connection();
        let actor =
            actor::get_actor_by_uri(&database, &uri).expect("Actor with this URI does not exist");

        if actor.modified.timestamp() <= expiration_time.timestamp() && !actor.local {
            println!("Refreshing actor {}", uri);

            match web::fetch_remote_object(&uri.to_string()) {
                Ok(object) => {
                    let parsed_object: serde_json::Value = serde_json::from_str(&object).unwrap();

                    match validator::validate_actor(parsed_object) {
                        Ok(actor) => {
                            let serialized_actor: Actor = serde_json::from_value(actor).unwrap();

                            actor::update(&database, serialized_actor.into());
                        }
                        Err(_) => {
                            eprintln!("Unable to refresh actor, remote object is invalid: {}", uri)
                        }
                    }
                }
                Err(_) => eprintln!("Unable to refresh actor: {}", uri),
            }
        }
    });
}

pub fn get_json_by_preferred_username(preferred_username: &str) -> serde_json::Value {
    let database = database::establish_connection();

    match actor::get_local_actor_by_preferred_username(&database, preferred_username) {
        Ok(actor) => {
            let actor: Actor = actor.into();

            return json!(actor);
        },
        Err(_) => json!({"error": "User not found."}),
    }
}


impl From<actor::Actor> for Actor {
    fn from(actor: actor::Actor) -> Self {
        let icon = match actor.icon {
            Some(url) => Some(serde_json::json!({"url": url, "type": "Image"})),
            None => None,
        };

        return Actor {
            context: Some(serde_json::json!([
            String::from("https://www.w3.org/ns/activitystreams"),
            String::from("https://w3id.org/security/v1"),
        ])),
            _type: String::from("Person"),
            id: actor.actor_uri.clone(),
            summary: actor.summary,
            following: format!("{}/following", &actor.actor_uri),
            followers: format!("{}/followers", &actor.actor_uri),
            inbox: format!("{}/inbox", &actor.actor_uri),
            outbox: format!("{}/outbox", &actor.actor_uri),
            preferredUsername: actor.preferred_username,
            name: actor.username,
            publicKey: serde_json::json!({
            "id": format!("{}#main-key", actor.actor_uri.clone()),
            "owner": actor.actor_uri.clone(),
            "publicKeyPem": actor.keys["public"].clone()
        }),
            url: actor.actor_uri,
            icon: icon,
            endpoints: Some(serde_json::json!({
            "sharedInbox":
                format!(
                    "{}://{}/inbox",
                    env::get_value(String::from("endpoint.base_scheme")),
                    env::get_value(String::from("endpoint.base_domain"))
                )
        })),
        }
    }
}

/// Creates an internal actor based on the ActivityPub actor
///
/// # Tests
///
/// Test for this function are in `tests/activitypub_actor.rs`
/// - create_internal_actor_with_empty_icon_url()
impl Into<actor::Actor> for Actor {
    fn into(self) -> actor::Actor {
        let icon = match self.icon {
            Some(icon) => {
                if !icon["url"].is_null() {
                    Some(icon["url"].as_str().unwrap().to_string())
                } else {
                    None
                }
            }
            None => None,
        };

        let inbox = match self.endpoints {
            Some(value) => {
                if value.get("sharedInbox").is_some() {
                    value["sharedInbox"].as_str().unwrap().to_string()
                } else {
                    self.inbox
                }
            }
            None => self.inbox
        };

        actor::Actor {
            id: 0,
            email: None,
            password: None,
            actor_uri: self.id,
            username: self.name,
            preferred_username: self.preferredUsername,
            summary: self.summary,
            inbox: Some(inbox),
            icon: icon,
            local: false,
            keys: serde_json::json!({"public" : self.publicKey["publicKeyPem"]}),
            followers: serde_json::json!({"activitypub": []}),
            created: Utc::now().naive_utc(),
            modified: Utc::now().naive_utc(),
        }
    }
}

pub struct Payload(serde_json::Value);

impl Activity {
    pub fn new(
        _type: &str,
        actor: &str,
        object: serde_json::Value,
        to: Vec<String>,
        cc: Vec<String>,
    ) -> Activity {
        return Activity {
            context: Some(serde_json::json!(vec![
                String::from("https://www.w3.org/ns/activitystreams"),
                String::from("https://w3id.org/security/v1"),
            ])),
            _type: _type.to_string(),
            id: format!(
                "{base_scheme}://{base_domain}/activities/{uuid}",
                base_scheme = env::get_value(String::from("endpoint.base_scheme")),
                base_domain = env::get_value(String::from("endpoint.base_domain")),
                uuid = Uuid::new_v4()
            ),
            actor: actor.to_string(),
            object: object,
            published: Utc::now().to_rfc3339().to_string(),
            to: to,
            cc: cc,
        };
    }
}

impl From<activity::Activity> for Activity {
    fn from(activity: activity::Activity) -> Self {
        return serde_json::from_value(activity.data)
            .expect("The data wasn't valid ActivityStreams2 data!");
    }
}

impl Into<activity::Activity> for Activity {
    fn into(self) -> activity::Activity {
        return activity::Activity {
            id: 0,
            data: serde_json::json!(&self),
            actor: self.actor,
        };
    }
}

impl Object {
    /// Returns a new ActivityStreams object of the type `Note`
    ///
    /// # Parameters
    ///
    /// * `actor`       -                   &str | Reference to an ActivityPub actor
    /// * `in_reply_to` -         Option<String> | An optional reference to another ActivityStreams object this object is a reply to
    /// * `content`     -                 &str   | The content of this note
    /// * `to`          -            Vec<String> | A vector of strings that provides direct receipients
    /// * `cc`          -            Vec<String> | A vector of strings that provides passive receipients
    /// * `tag`         - Vec<serde_json::Value> | A vector of tags to ActivityStreams objects wrapped in JSON
    ///
    pub fn note(
        actor: &str,
        in_reply_to: Option<String>,
        content: &str,
        to: Vec<String>,
        cc: Vec<String>,
        tag: Vec<serde_json::Value>,
    ) -> Object {
        return Object {
            context: Some(serde_json::json!(vec![
                String::from("https://www.w3.org/ns/activitystreams"),
                String::from("https://w3id.org/security/v1"),
            ])),
            _type: String::from("Note"),
            id: format!(
                "{base_scheme}://{base_domain}/objects/{uuid}",
                base_scheme = env::get_value(String::from("endpoint.base_scheme")),
                base_domain = env::get_value(String::from("endpoint.base_domain")),
                uuid = Uuid::new_v4()
            ),
            attributedTo: actor.to_string(),
            inReplyTo: in_reply_to,
            summary: None,
            content: content.to_string(),
            published: Utc::now().to_rfc3339().to_string(),
            to: to,
            cc: cc,
            tag: Some(tag),
            attachment: None,
            sensitive: Some(false),
        };
    }
}

impl FromDataSimple for Payload {
    type Error = String;

    fn from_data(req: &Request, data: Data) -> data::Outcome<Self, String> {
        let mut data_stream = String::new();

        // Read at most a 1MB payload
        //
        // TODO: This value should be adjustable in the config
        if let Err(e) = data.open().take(1048576).read_to_string(&mut data_stream) {
            return Outcome::Failure((Status::InternalServerError, format!("{:?}", e)));
        }

        match serde_json::from_str(&data_stream) {
            Ok(value) => return Outcome::Success(Payload(value)),
            Err(e) => return Outcome::Failure((Status::UnprocessableEntity, format!("{:?}", e))),
        }
    }
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
            let route = request.route().unwrap().to_string();
            let request_target: Vec<&str> = route.split_whitespace().collect();

            return Outcome::Success(Signature {
                algorithm: None,
                content_length: Some(content_length_vec.get(0).unwrap_or_else(|| &"").to_string()),
                date: date_vec.get(0).unwrap_or_else(|| &"").to_string(),
                digest: Some(digest_vec.get(0).unwrap_or_else(|| &"").to_string()),
                headers: headers.iter().map(|header| header.to_string()).collect(),
                host: host_vec.get(0).unwrap_or_else(|| &"").to_string(),
                key_id: None,
                request_target: Some(request_target[1].to_string()),
                signature: String::new(),
                signature_in_bytes: Some(
                    base64::decode(&parsed_signature["signature"].to_owned().into_bytes()).unwrap(),
                ),
            });
        }
    }
}
