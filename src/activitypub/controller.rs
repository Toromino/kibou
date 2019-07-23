use activity::get_ap_activity_by_id;
use activity::get_ap_object_by_id;
use activity::insert_activity;
use activitypub::activity::create_internal_activity;
use activitypub::Activity;
use activitypub::Object;
use activitypub::actor::add_follow;
use activitypub::actor::create_internal_actor;
use activitypub::actor::remove_follow;
use activitypub::actor::Actor;
use activitypub::validator;
use actor::create_actor;
use actor::get_actor_by_uri;
use actor::is_actor_followed_by;
use chrono::Utc;
use database;
use env;
use notification::{self, Notification};
use std::thread;
use url::Url;
use uuid::Uuid;
use web;
use web::http_signatures::Signature;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use database::PooledConnection;

pub fn activity_build(
    _type: &str,
    actor: &str,
    object: serde_json::Value,
    to: Vec<String>,
    cc: Vec<String>,
) -> Activity {
    let database = database::establish_connection();
    let new_activity = Activity {
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

    insert_activity(
        &database,
        create_internal_activity(&serde_json::json!(&new_activity), &new_activity.actor),
    );
    new_activity
}

pub fn activity_by_id(pooled_connection: &PooledConnection, id: &str) -> JsonValue {
    let activity_id = format!(
        "{}://{}/activities/{}",
        env::get_value(String::from("endpoint.base_scheme")),
        env::get_value(String::from("endpoint.base_domain")),
        id
    );

    match get_ap_activity_by_id(pooled_connection, &activity_id) {
        Ok(activity) => json!(serialize_from_internal_activity(activity).object),
        Err(_) => json!({"error": "Object not found."}),
    }
}

/// Determines whether an ActivityPub actor exists in the database
///
/// # Parameters
///
/// * `actor_id` - &str | Reference to an ActivityPub actor
///
/// # Tests
///
/// Tests for this function are in `tests/activitypub_controller.rs`
/// - actor_exists()
/// - err_actor_exists()
pub fn actor_exists(actor_id: &str) -> bool {
    let database = database::establish_connection();

    match get_actor_by_uri(&database, actor_id) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Returns a new ActivityStreams object of the type `Note`
///
/// # Parameters
///
/// * `actor`    -                   &str | Reference to an ActivityPub actor
/// * `reply_to` -         Option<String> | An optional reference to another ActivityStreams object this object is a reply to
/// * `content`  -                 String | The content of this note
/// * `to`       -            Vec<String> | A vector of strings that provides direct receipients
/// * `cc`       -            Vec<String> | A vector of strings that provides passive receipients
/// * `tag`      - Vec<serde_json::Value> | A vector of tags to ActivityStreams objects wrapped in JSON
///
pub fn note(
    actor: &str,
    reply_to: Option<String>,
    content: String,
    to: Vec<String>,
    cc: Vec<String>,
    tag: Vec<serde_json::Value>,
) -> Object {
    Object {
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
        inReplyTo: reply_to,
        summary: None,
        content: content,
        published: Utc::now().to_rfc3339().to_string(),
        to: to,
        cc: cc,
        tag: Some(tag),
        attachment: None,
        sensitive: Some(false),
    }
}

/// Trys to fetch a remote object based on the ActivityStreams id
///
/// # Description
///
/// If the URL was successfully parsed, this function will try to fetch the remote object and
/// determine whether it's a known and valid ActivityStreams object or ActivityPub actor.
///
/// # Parameters
///
/// * `url` - String | Link to an ActivityStreams object
///
pub fn fetch_object_by_id(url: String) {
    let mut parsed_url = String::new();
    let stripped_characters = "\"";
    for character in url.chars() {
        if !stripped_characters.contains(character) {
            parsed_url.push(character);
        }
    }
    match Url::parse(&parsed_url) {
        Ok(remote_url) => {
            if !object_exists(&remote_url.to_string()) && !actor_exists(&remote_url.to_string()) {
                println!("Trying to fetch document: {}", &url);
                match web::fetch_remote_object(&remote_url.to_string()) {
                    Ok(object) => {
                        let parsed_object: serde_json::Value =
                            serde_json::from_str(&object).unwrap();

                        match validator::validate_object(parsed_object.clone(), false) {
                            Ok(as2_object) => {
                                handle_object(as2_object);
                                println!("Successfully fetched object: {}", &url);
                            }
                            Err(_) => (),
                        }

                        match validator::validate_actor(parsed_object.clone()) {
                            Ok(as2_actor) => {
                                handle_actor(as2_actor);
                                println!("Successfully fetched actor: {}", &url);
                            }
                            Err(_) => (),
                        }
                    }
                    Err(_) => eprintln!("Unable to fetch document: {}", &url),
                }
            }
        }
        Err(_) => (),
    }
}

/// Determines whether an ActivityStreams object exists in the database
///
/// # Parameters
///
/// * `object_id` - &str | Reference to an ActivityStreams object
///
/// # Tests
///
/// Tests for this function are in `tests/activitypub_controller.rs`
/// - object_exists()
/// - err_object_exists()
pub fn object_exists(object_id: &str) -> bool {
    let database = database::establish_connection();

    match get_ap_object_by_id(&database, object_id) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Handles incoming requests of the inbox
///
/// # Parameters
///
/// * `activity`  - serde_json::Value           | An ActivityStreams activity serialized in JSON
/// * `signature` - activitiypub::HTTPSignature | The activity's signature, signed by an actor
///
/// # Tests
///
/// [TODO]
pub fn prepare_incoming(activity: serde_json::Value, signature: Signature) {
    match validator::validate_activity(activity.clone(), signature) {
        Ok(sanitized_activity) => handle_activity(sanitized_activity),
        Err(_) => eprintln!("Validation failed for activity: {:?}", activity),
    }
}

/// Handles a newly fetched object and wraps it into it's own internal `Create` activity
///
/// # Parameters
///
/// * `object` - serde_json::Value | An ActivityStreams object serialized in JSON
///
/// # Tests
///
/// [TODO]
fn handle_object(object: serde_json::Value) {
    let serialized_object: Object = serde_json::from_value(object.clone()).unwrap();

    if !serialized_object.inReplyTo.is_none() {
        if !object_exists(&serialized_object.inReplyTo.clone().unwrap()) {
            fetch_object_by_id(serialized_object.inReplyTo.unwrap().to_string());
        }
    }

    if !object_exists(&serialized_object.id) {
        // Wrapping new object in an activity, as raw objects don't get stored
        let _activity = activity_build(
            "Create",
            &serialized_object.attributedTo,
            object,
            serialized_object.to,
            serialized_object.cc,
        );
    }
}

/// Handles a newly fetched actor
///
/// # Parameters
///
/// * `actor` - serde_json::Value | An ActivityPub actor serialized in JSON
///
/// # Tests
///
/// [TODO]
fn handle_actor(actor: serde_json::Value) {
    let database = database::establish_connection();
    let serialized_actor: Actor = serde_json::from_value(actor).unwrap();

    create_actor(&database, &mut create_internal_actor(serialized_actor));
}

/// Final handling of incoming ActivityStreams activities which have already been validated
///
/// # Parameters
///
/// * `activity` - serde_json::Value | An ActivityStreams activity serialized in JSON
///
/// # Tests
///
/// [TODO]
fn handle_activity(activity: serde_json::Value) {
    let database = database::establish_connection();
    let actor = activity["actor"].as_str().unwrap().to_string();

    match activity["type"].as_str() {
        Some("Accept") => {
            let mut activity_id: &str = "";

            if activity["object"].is_string() {
                activity_id = activity["object"].as_str().unwrap();
            } else if activity["object"].is_object() {
                activity_id = activity["object"]["id"].as_str().unwrap();
            }

            match get_ap_activity_by_id(&database, activity_id) {
                Ok(original_activity) => match original_activity.data["type"].as_str().unwrap() {
                    "Follow" => {
                        let sender = original_activity.data["actor"].as_str().unwrap();
                        let receipient_actor = get_actor_by_uri(
                            &database,
                            original_activity.data["object"].as_str().unwrap(),
                        )
                        .unwrap();

                        match is_actor_followed_by(&database, &receipient_actor, sender) {
                            Ok(false) => {
                                add_follow(&receipient_actor.actor_uri, sender, activity_id)
                            }
                            Ok(true) => (),
                            Err(e) => eprintln!("{}", e),
                        }

                        insert_activity(&database, create_internal_activity(&activity, &actor));
                    }
                    &_ => (),
                },
                Err(e) => eprintln!("Unknown object mentioned in `Accept` activity {}", e),
            }

            insert_activity(&database, create_internal_activity(&activity, &actor));
        }
        Some("Announce") => {
            let object_id = activity["object"].as_str().unwrap().to_string();
            thread::spawn(move || {
                fetch_object_by_id(object_id);
            });

            let id = insert_activity(&database, create_internal_activity(&activity, &actor)).id;

            match get_actor_by_uri(&database, &actor) {
                Ok(actor) => {
                    if actor.local {
                        let notification = Notification::new(id, actor.id);
                        notification::insert(&database, notification);
                    }
                }

                Err(_) => (),
            }
        }
        Some("Create") => {
            if activity["object"].get("inReplyTo").is_some() {
                if activity["object"]["inReplyTo"] != serde_json::Value::Null {
                    let reply_id = activity["object"]["inReplyTo"]
                        .as_str()
                        .unwrap()
                        .to_string();

                    let id =
                        insert_activity(&database, create_internal_activity(&activity, &actor)).id;

                    match get_ap_object_by_id(&database, &reply_id) {
                        Ok(object) => match get_actor_by_uri(&database, &object.actor) {
                            Ok(actor) => {
                                if actor.local {
                                    let notification = Notification::new(id, actor.id);
                                    notification::insert(&database, notification);
                                }
                            }
                            Err(_) => eprintln!(
                                "Error: Activity '{}' was fetched without a valid actor!",
                                &reply_id
                            ),
                        },
                        Err(_) => {
                            thread::spawn(move || {
                                fetch_object_by_id(reply_id);
                            });
                        }
                    }
                }
            }
        }
        Some("Follow") => {
            let remote_account = get_actor_by_uri(&database, &actor).unwrap();

            let id = insert_activity(&database, create_internal_activity(&activity, &actor)).id;

            match get_actor_by_uri(&database, activity["object"].as_str().unwrap()) {
                Ok(actor) => {
                    if actor.local {
                        let notification = Notification::new(id, actor.id);
                        notification::insert(&database, notification);

                        match is_actor_followed_by(&database, &actor, &remote_account.actor_uri) {
                            Ok(false) => {
                                let accept_activity = serde_json::to_value(activity_build(
                                    "Accept",
                                    &actor.actor_uri,
                                    activity["id"].clone(),
                                    vec![remote_account.actor_uri.clone()],
                                    vec![],
                                ))
                                .unwrap();

                                add_follow(
                                    &actor.actor_uri,
                                    &remote_account.actor_uri,
                                    activity["id"].as_str().unwrap(),
                                );
                                web::federator::enqueue(
                                    actor,
                                    accept_activity,
                                    vec![remote_account.inbox.unwrap()],
                                );
                            }

                            // *Note*
                            //
                            // Kibou should still send a `Accept` activity even if one was already sent, in
                            // case the original `Accept` activity did not reach the remote server.
                            Ok(true) => {
                                let accept_activity = serde_json::to_value(activity_build(
                                    "Accept",
                                    &actor.actor_uri,
                                    activity["id"].clone(),
                                    vec![remote_account.actor_uri],
                                    vec![],
                                ))
                                .unwrap();
                                web::federator::enqueue(
                                    actor,
                                    accept_activity,
                                    vec![remote_account.inbox.unwrap()],
                                );
                            }
                            Err(_) => (),
                        }
                    }
                }

                Err(_) => (),
            }
        }
        Some("Like") => {
            let object_id = activity["object"].as_str().unwrap().to_string();
            thread::spawn(move || {
                fetch_object_by_id(object_id);
            });

            let id = insert_activity(&database, create_internal_activity(&activity, &actor)).id;

            match get_actor_by_uri(&database, &actor) {
                Ok(actor) => {
                    if actor.local {
                        let notification = Notification::new(id, actor.id);
                        notification::insert(&database, notification);
                    }
                }

                Err(_) => (),
            }
        }
        Some("Undo") => {
            let remote_account = get_actor_by_uri(&database, &actor).unwrap();
            let object =
                get_ap_activity_by_id(&database, activity["object"]["id"].as_str().unwrap())
                    .unwrap();

            match object.data["type"].as_str().unwrap() {
                "Follow" => {
                    let account =
                        get_actor_by_uri(&database, object.data["object"].as_str().unwrap())
                            .unwrap();

                    match is_actor_followed_by(&database, &account, &actor) {
                        Ok(true) => remove_follow(&account.actor_uri, &remote_account.actor_uri),
                        Ok(false) => (),
                        Err(_) => (),
                    }

                    insert_activity(&database, create_internal_activity(&activity, &actor));
                }
                &_ => (),
            }
        }
        _ => (),
    }
}
