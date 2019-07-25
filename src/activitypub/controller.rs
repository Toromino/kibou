use activity::get_ap_activity_by_id;
use activity::get_ap_object_by_id;
use activity::insert_activity;
use activitypub::add_follow;
use activitypub::remove_follow;
use activitypub::Actor;
use activitypub::validator;
use activitypub::Activity;
use activitypub::Object;
use actor;
use actor::get_actor_by_uri;
use actor::is_actor_followed_by;
use chrono::Utc;
use database;
use database::{Pool, PooledConnection};
use env;
use notification::{self, Notification};
use rocket::http::Status;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use std::thread;
use url::Url;
use uuid::Uuid;
use web;
use web::http_signatures::Signature;

pub fn activity_by_id(pooled_connection: &PooledConnection, id: &str) -> JsonValue {
    let activity_id = format!(
        "{}://{}/activities/{}",
        env::get_value(String::from("endpoint.base_scheme")),
        env::get_value(String::from("endpoint.base_domain")),
        id
    );

    match get_ap_activity_by_id(pooled_connection, &activity_id) {
        Ok(activity) => json!(Activity::from(activity)),
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
pub fn actor_exists(pooled_connection: &PooledConnection, uri: &str) -> bool {
    match get_actor_by_uri(pooled_connection, uri) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn object_by_id(pooled_connection: &PooledConnection, id: &str) -> JsonValue {
    let object_id = format!(
        "{}://{}/objects/{}",
        env::get_value(String::from("endpoint.base_scheme")),
        env::get_value(String::from("endpoint.base_domain")),
        id
    );

    match get_ap_object_by_id(pooled_connection, &object_id) {
        Ok(activity) => json!(Activity::from(activity).object),
        Err(_) => json!({"error": "Object not found."}),
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

/// Tries to fetch a remote object based on the ActivityStreams id
///
/// # Description
///
/// If the URL was successfully parsed, this function will try to fetch the remote object and
/// determine whether it's a known and valid ActivityStreams object or ActivityPub actor.
///
/// # Parameters
///
/// * `url` - &str | Link to an ActivityStreams object
///
pub fn fetch(pooled_connection: &PooledConnection, url: &str) {
    match Url::parse(url) {
        Ok(remote_url) => {
            if !object_exists(&remote_url.to_string())
                && !actor_exists(pooled_connection, &remote_url.to_string())
            {
                println!("Trying to fetch document: {}", &url);
                match web::fetch_remote_object(&remote_url.to_string()) {
                    Ok(object) => {
                        let parsed_object: serde_json::Value =
                            serde_json::from_str(&object).unwrap();

                        match validator::validate_object(pooled_connection, parsed_object.clone(), false) {
                            Ok(as2_object) => {
                                handle(pooled_connection, as2_object);
                                println!("Successfully fetched object: {}", &url);
                            }
                            Err(_) => (),
                        }

                        match validator::validate_actor(parsed_object.clone()) {
                            Ok(as2_actor) => {
                                handle(pooled_connection, as2_actor);
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

/// Handles incoming requests of the inbox
///
/// # Parameters
///
/// * `activity`  - serde_json::Value           | An ActivityStreams activity serialized in JSON
/// * `signature` - activitiypub::Signature | The activity's signature, signed by an actor
///
pub fn validate_incoming(pooled_connection: &PooledConnection, activity: serde_json::Value, signature: Signature) -> Status {
    match validator::validate_activity(pooled_connection, activity.clone(), signature) {
        Ok(sanitized_activity) => {
            handle(pooled_connection, sanitized_activity);
            return Status::Ok;
        },
        Err(_) => {
            eprintln!("Validation failed for activity: {:?}", activity);
            return Status::InternalServerError;
        },
    }
}

fn generate_notifications(
    pooled_connection: &PooledConnection,
    activity_id: i64,
    collection: Vec<String>,
) {
    for person in collection {
        match get_actor_by_uri(pooled_connection, &person) {
            Ok(actor) => {
                if actor.local {
                    notification::insert(
                        pooled_connection,
                        Notification::new(activity_id, actor.id),
                    );
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }
}

/// Handles validated ActivityStreams data
///
/// # Parameters
///
/// * `payload` - serde_json::Value | An ActivityStreams payload
fn handle(pooled_connection: &PooledConnection, payload: serde_json::Value) {
    match payload["type"].as_str().unwrap() {
        "Person" => {
            let actor: Actor = serde_json::from_value(payload).unwrap();
            actor::create_actor(pooled_connection, &mut actor.into());
        }
        "Note" => {
            let object: Object = serde_json::from_value(payload).unwrap();

            match &object.inReplyTo {
                Some(reference) => fetch(pooled_connection, reference),
                None => (),
            }

            // Raw ActivityStreams notes do not get stored,
            // so Kibou wraps it in a new 'Create' activity
            if !object_exists(&object.id) {
                let wrapped_object = Activity::new(
                    "Create",
                    &object.attributedTo,
                    serde_json::json!(object),
                    object.to,
                    object.cc,
                );

                insert_activity(pooled_connection, wrapped_object.into());
            }
        }
        "Accept" => {
            let activity: Activity = serde_json::from_value(payload).unwrap();
            let object: &str = if activity.object.is_string() {
                activity.object.as_str().unwrap()
            } else {
                activity.object["id"].as_str().unwrap()
            };

            match get_ap_activity_by_id(pooled_connection, &object) {
                Ok(referenced_activity) => {
                    let referenced_activity = Activity::from(referenced_activity);
                    match referenced_activity._type.as_str() {
                        "Follow" => {
                            let actor = referenced_activity.object.as_str().unwrap();
                            match is_actor_followed_by(
                                pooled_connection,
                                actor,
                                &referenced_activity.actor,
                            ) {
                                Ok(false) => {
                                    add_follow(
                                        actor,
                                        &referenced_activity.actor,
                                        &referenced_activity.id,
                                    );
                                    insert_activity(pooled_connection, activity.into());
                                }
                                Ok(true) => (),
                                Err(e) => eprintln!("{}", e),
                            }
                        }
                        _ => (),
                    }
                }
                Err(_) => eprintln!("Unknown reference in 'Accept' activity {}", &activity.id),
            }
        }
        "Announce" => {
            let activity: Activity = serde_json::from_value(payload).unwrap();
            let object = activity.object.as_str().unwrap();
            let id = insert_activity(pooled_connection, activity.clone().into()).id;

            fetch(pooled_connection, object);
            generate_notifications(pooled_connection, id, activity.to);
        }
        "Create" => {
            let activity: Activity = serde_json::from_value(payload).unwrap();
            let object: Object = serde_json::from_value(activity.clone().object).unwrap();
            let id = insert_activity(pooled_connection, activity.clone().into()).id;

            match object.inReplyTo {
                Some(reference) => fetch(pooled_connection, &reference),
                None => (),
            }

            generate_notifications(pooled_connection, id, activity.to);
        }
        "Follow" => {
            let activity: Activity = serde_json::from_value(payload).unwrap();
            let actor = get_actor_by_uri(pooled_connection, &activity.actor).unwrap();
            let followee = activity.object.as_str().unwrap();

            match get_actor_by_uri(pooled_connection, followee) {
                Ok(followee_actor) => {
                    if followee_actor.local {
                        match is_actor_followed_by(pooled_connection, followee, &actor.actor_uri) {
                            Ok(false) => {
                                insert_activity(pooled_connection, activity.clone().into());
                                add_follow(followee, &actor.actor_uri, &activity.id);
                                let accept_activity = Activity::new(
                                    "Accept",
                                    followee,
                                    serde_json::json!(activity.id),
                                    vec![actor.actor_uri],
                                    Vec::new(),
                                );
                                let id =
                                    insert_activity(pooled_connection, accept_activity.clone().into()).id;

                                notification::insert(
                                    pooled_connection,
                                    Notification::new(id, followee_actor.id),
                                );
                                web::federator::enqueue(
                                    followee_actor,
                                    serde_json::json!(accept_activity),
                                    vec![actor.inbox.unwrap()],
                                );
                            }
                            // The remote server might not know that their actor is already following (unsynchronized state),
                            // so even if the followee is already being followed, accept the 'Follow' activity and send it to the actor.
                            Ok(true) => {
                                insert_activity(pooled_connection, activity.clone().into());
                                let accept_activity = Activity::new(
                                    "Accept",
                                    followee,
                                    serde_json::json!(activity.id),
                                    vec![actor.actor_uri],
                                    Vec::new(),
                                );
                                let id =
                                    insert_activity(pooled_connection, accept_activity.clone().into()).id;

                                notification::insert(
                                    pooled_connection,
                                    Notification::new(id, followee_actor.id),
                                );
                                web::federator::enqueue(
                                    followee_actor,
                                    serde_json::json!(accept_activity),
                                    vec![actor.inbox.unwrap()],
                                );
                            }
                            Err(e) => eprintln!("{}", e),
                        }
                    }
                }
                Err(e) => eprintln!("{}", e),
            }
        }
        "Like" => {
            let activity: Activity = serde_json::from_value(payload).unwrap();
            let object = activity.object.as_str().unwrap();
            let id = insert_activity(pooled_connection, activity.clone().into()).id;

            fetch(pooled_connection, object);

            generate_notifications(pooled_connection, id, activity.to);
        }
        "Undo" => {
            let activity: Activity = serde_json::from_value(payload).unwrap();
            let actor = &activity.actor;

            let object: Activity = if activity.object.is_object() {
                serde_json::from_value(activity.clone().object).unwrap()
            } else {
                get_ap_activity_by_id(pooled_connection, activity.object.as_str().unwrap()).expect(
                    &format!(
                        "A referenced object within an 'Undo' activity should exist! ({})",
                        &activity.id
                    ),
                ).into()
            };

            let followee = object.object.as_str().unwrap();

            // The actor of the 'Undo' activity must be the same as the actor of the
            // original follow activity, otherwise an unrelated actor could mutate
            // the follow relationship between these actors.
            //
            // This should be moved into activitypub::validator
            if followee == actor {
                match object._type.as_str() {
                    "Follow" => {
                        match is_actor_followed_by(pooled_connection, followee, &object.actor) {
                            Ok(true) => {
                                remove_follow(followee, &object.actor);
                                insert_activity(pooled_connection, activity.into());
                            }
                            Ok(false) => (),
                            Err(e) => eprintln!("{}", e),
                        }
                    }
                    _ => ()
                }
            }
        }
        _ => (),
    }
}
