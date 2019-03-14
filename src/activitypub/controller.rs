use actor::create_actor as create_actor;
use actor::get_actor_by_uri;
use actor::is_actor_followed_by;
use activity::get_ap_activity_by_id;
use activity::get_ap_object_by_id;
use activity::insert_activity;
use activitypub::actor::Actor;
use activitypub::actor::add_follow;
use activitypub::actor::remove_follow;
use activitypub::actor::create_internal_actor;
use activitypub::activity::Activity;
use activitypub::activity::Object;
use activitypub::activity::create_internal_activity;
use activitypub::validator;
use chrono::Utc;
use database;
use env;
use url::Url;
use uuid::Uuid;
use web_handler;

pub fn activity_accept(_actor: &str, _object: &str) -> Activity
{
    let database = database::establish_connection();
    let new_activity = Activity
    {
        context: vec![String::from("https://www.w3.org/ns/activitystreams"), String::from("https://w3id.org/security/v1")],
        _type: String::from("Accept"),
        id: format!("{base_scheme}://{base_domain}/activities/{uuid}",
        base_scheme = env::get_value(String::from("endpoint.base_scheme")),
        base_domain = env::get_value(String::from("endpoint.base_domain")),
        uuid = Uuid::new_v4()),
        actor: _actor.to_string(),
        object: serde_json::json!(_object),
        published: Utc::now().to_rfc3339().to_string(),
        to: vec![],
        cc: vec![]
    };

    insert_activity(&database, create_internal_activity(serde_json::json!(&new_activity), new_activity.actor.clone()));
    new_activity
}

/// Creates a new activity, inserts it into the database and returns the newly created activity
///
/// # Parameters
///
/// * `actor` - String | Reference to an actor by their actor_uri
/// * `object` - serde_json::Value | An ActivityStreams object serialized in JSON
/// * `to` - Vec<String> | A vector of strings that provides direct receipients
/// * `cc` - Vec<String> | A vector of strings that provides passive receipients
///
pub fn activity_create(_actor: &str, _object: serde_json::Value, _to: Vec<String>, _cc: Vec<String>) -> Activity
{
    let database = database::establish_connection();
    let new_activity = Activity
    {
        context: vec![String::from("https://www.w3.org/ns/activitystreams"), String::from("https://w3id.org/security/v1")],
        _type: String::from("Create"),
        id: format!("{base_scheme}://{base_domain}/activities/{uuid}",
        base_scheme = env::get_value(String::from("endpoint.base_scheme")),
        base_domain = env::get_value(String::from("endpoint.base_domain")),
        uuid = Uuid::new_v4()),
        actor: _actor.to_string(),
        object: _object,
        published: Utc::now().to_rfc3339().to_string(),
        to: _to,
        cc: _cc
    };

    insert_activity(&database, create_internal_activity(serde_json::json!(&new_activity), new_activity.actor.clone()));
    new_activity
}

pub fn activity_follow(_actor: &str, _object: String) -> Activity
{
    let database = database::establish_connection();
    let new_activity = Activity
    {
        context: vec![String::from("https://www.w3.org/ns/activitystreams"), String::from("https://w3id.org/security/v1")],
        _type: String::from("Follow"),
        id: format!("{base_scheme}://{base_domain}/activities/{uuid}",
        base_scheme = env::get_value(String::from("endpoint.base_scheme")),
        base_domain = env::get_value(String::from("endpoint.base_domain")),
        uuid = Uuid::new_v4()),
        actor: _actor.to_string(),
        object: serde_json::json!(_object),
        published: Utc::now().to_rfc3339().to_string(),
        to: vec![_object],
        cc: vec![]
    };

    insert_activity(&database, create_internal_activity(serde_json::json!(&new_activity), new_activity.actor.clone()));
    new_activity
}

pub fn activity_like(_actor: &str, _object: String, _to: Vec<String>, _cc: Vec<String>) -> Activity
{
    let database = database::establish_connection();
    let new_activity = Activity
    {
        context: vec![String::from("https://www.w3.org/ns/activitystreams"), String::from("https://w3id.org/security/v1")],
        _type: String::from("Like"),
        id: format!("{base_scheme}://{base_domain}/activities/{uuid}",
        base_scheme = env::get_value(String::from("endpoint.base_scheme")),
        base_domain = env::get_value(String::from("endpoint.base_domain")),
        uuid = Uuid::new_v4()),
        actor: _actor.to_string(),
        object: serde_json::json!(_object),
        published: Utc::now().to_rfc3339().to_string(),
        to: _to,
        cc: _cc
    };

    insert_activity(&database, create_internal_activity(serde_json::json!(&new_activity), new_activity.actor.clone()));
    new_activity
}

pub fn note(actor: &str, reply_to: Option<String>, _content: String, _to: Vec<String>, _cc: Vec<String>, _tag: Vec<serde_json::Value>) -> Object
{
    Object
    {
        _type: String::from("Note"),
        id: format!("{base_scheme}://{base_domain}/objects/{uuid}",
        base_scheme = env::get_value(String::from("endpoint.base_scheme")),
        base_domain = env::get_value(String::from("endpoint.base_domain")),
        uuid = Uuid::new_v4()),
        attributedTo: actor.to_string(),
        inReplyTo: reply_to,
        summary: None, // [TODO]
        content: _content,
        published: Utc::now().to_rfc3339().to_string(),
        to: _to,
        cc: _cc,
        tag: _tag
    }
}

/// # Tests
///
/// [TODO]
pub fn fetch_object_by_id(url: String)
{
    let mut parsed_url = String::new();
    let stripped_characters = "\"";
    for character in url.chars()
    {
        if !stripped_characters.contains(character)
        {
            parsed_url.push(character);
        }
    }
    match Url::parse(&parsed_url)
    {
        Ok(remote_url) =>
        {
            if !object_exists(&remote_url.to_string()) && !actor_exists(&remote_url.to_string())
            {
                println!("Trying to fetch document: {}", &url);
                match web_handler::fetch_remote_object(&remote_url.to_string())
                {
                    Ok(object) =>
                    {
                        let parsed_object: serde_json::Value = serde_json::from_str(&object).unwrap();
                        if validator::validate_object(parsed_object.clone()).is_ok()
                        {
                            println!("Successfully fetched object: {}", &url);
                            handle_object(parsed_object.clone());
                        }
                        else if validator::validate_actor(parsed_object.clone()).is_ok()
                        {
                            println!("Successfully fetched actor: {}", &url);
                            handle_actor(parsed_object.clone());
                        }
                        else { eprintln!("Unable to validate fetched document: {}", &url); }
                    },

                    Err(_) => eprintln!("Unable to fetch document: {}", &url)
                }
            }
        },
        Err(_) => ()
    }
}

/// # Tests
///
/// [TODO]
pub fn prepare_incoming(object: serde_json::Value)
{
    let object_string = &object.to_string();

    match validator::validate_activity(object)
    {
        Ok(sanitized_activity) => handle_activity(sanitized_activity),
        Err(_) => eprintln!("Validation failed for activity: {}", object_string)
    }
}

/// # Tests
///
/// Tests for this function are in `tests/activitypub_controller.rs`
/// - actor_exists()
/// - err_actor_exists()
pub fn actor_exists(actor_id: &str) -> bool
{
    let database = database::establish_connection();

    match get_actor_by_uri(&database, actor_id)
    {
        Ok(_) => true,
        Err(_) => false
    }
}

/// # Tests
///
/// Tests for this function are in `tests/activitypub_controller.rs`
/// - object_exists()
/// - err_object_exists()
pub fn object_exists(object_id: &str) -> bool
{
    let database = database::establish_connection();

    match get_ap_object_by_id(&database, object_id)
    {
        Ok(_) => true,
        Err(_) => false
    }
}

/// # Tests
///
/// [TODO]
fn resolve_participants(participants: Vec<String>)
{
    // Resolve all participants
    for participant in participants.iter()
    {
        if participant != "" && !actor_exists(&participant.to_string())
        {
            fetch_object_by_id(participant.to_string())
        }
    }
}

/// Resolve threads
///
/// # Tests
///
/// [TODO]
fn resolve_thread(mentioned_objects: Vec<String>)
{
    for object in mentioned_objects.iter()
    {
        if !object_exists(&object.to_string())
        {
            fetch_object_by_id(object.to_string());
        }
    }
}

/// # Tests
///
/// [TODO]
fn handle_object(object: serde_json::Value)
{
    let serialized_object: Object = serde_json::from_value(object.clone()).unwrap();
    let participants = vec![serialized_object.attributedTo.clone()];
    let mut mentioned_objects = vec![];

    if !serialized_object.inReplyTo.is_none()
    {
        mentioned_objects.push(serialized_object.inReplyTo.unwrap());
    }

    // Wrapping new object in an activity, as raw objects don't get stored
    let activity = activity_create(&serialized_object.attributedTo, object, serialized_object.to, serialized_object.cc);

    if !mentioned_objects.is_empty() { resolve_thread(mentioned_objects); }
    if !participants.is_empty() { resolve_participants(participants); }
}

/// # Tests
///
/// [TODO]
fn handle_actor(actor: serde_json::Value)
{
    let database = database::establish_connection();
    let serialized_actor: Actor = serde_json::from_value(actor).unwrap();

    create_actor(&database, &mut create_internal_actor(serialized_actor));
}

/// # Tests
///
/// [TODO]
fn handle_activity(activity: serde_json::Value)
{
    let database = database::establish_connection();
    let actor = activity["actor"].as_str().unwrap().to_string();
    let mut participants = vec![];
    let mut mentioned_objects = vec![];

    match activity["type"].as_str() {
        Some("Create") => {
            participants.push(activity["actor"].as_str().unwrap().to_string());
            participants.push(activity["object"]["attributedTo"].as_str().unwrap().to_string());

            if activity["object"].get("inReplyTo").is_some()
            {
                if activity["object"]["inReplyTo"] != serde_json::Value::Null
                {
                    mentioned_objects.push(activity["object"]["inReplyTo"].as_str().unwrap().to_string());
                }
            }

            insert_activity(&database, create_internal_activity(activity, actor));
        },
        Some("Like") => {
            participants.push(activity["actor"].as_str().unwrap().to_string());
            mentioned_objects.push(activity["object"].as_str().unwrap().to_string());

            insert_activity(&database, create_internal_activity(activity, actor));
        },
        Some("Announce") => {
            participants.push(activity["actor"].as_str().unwrap().to_string());
            mentioned_objects.push(activity["object"].as_str().unwrap().to_string());

            insert_activity(&database, create_internal_activity(activity, actor));
        },
        Some("Follow") => {
            if !actor_exists(activity["actor"].as_str().unwrap())
            {
                fetch_object_by_id(activity["actor"].as_str().unwrap().to_string());
            }

            let remote_account = get_actor_by_uri(&database, activity["actor"].as_str().unwrap()).unwrap();
            let account = get_actor_by_uri(&database, activity["object"].as_str().unwrap()).unwrap();

            match is_actor_followed_by(&database, &account, activity["actor"].as_str().unwrap())
            {
                Ok(false) => {
                    let new_activity = serde_json::to_value(activity_accept(&account.actor_uri, activity["id"].as_str().unwrap())).unwrap();

                    add_follow(&account.actor_uri, &remote_account.actor_uri);
                    web_handler::federator::enqueue(account, new_activity, vec![remote_account.inbox.unwrap()]);
                }

                // *Note*
                //
                // Kibou should still send a `Accept` activity even if one was already sent, in
                // case the original `Accept` activity did not reach the remote server.
                Ok(true) => {
                    let new_activity = serde_json::to_value(activity_accept(&account.actor_uri, activity["id"].as_str().unwrap())).unwrap();
                    web_handler::federator::enqueue(account, new_activity, vec![remote_account.inbox.unwrap()]);
                },
                Err(_) => ()
            }

            insert_activity(&database, create_internal_activity(activity, actor));
        },
        Some("Undo") => {
            let remote_account = get_actor_by_uri(&database, activity["actor"].as_str().unwrap()).unwrap();
            let object = get_ap_activity_by_id(&database, activity["object"]["id"].as_str().unwrap()).unwrap();

            match object.data["type"].as_str().unwrap()
            {
                "Follow" =>
                {
                    let account = get_actor_by_uri(&database, object.data["object"].as_str().unwrap()).unwrap();

                    match is_actor_followed_by(&database, &account, activity["actor"].as_str().unwrap())
                    {
                        Ok(true) => remove_follow(&account.actor_uri, &remote_account.actor_uri),
                        Ok(false) => (),
                        Err(_) => ()
                    }

                    insert_activity(&database, create_internal_activity(activity.clone(), actor.clone()));
                },
                &_ => ()
            }
        }
        _ => ()
    }

    if !mentioned_objects.is_empty() { resolve_thread(mentioned_objects); }
    if !participants.is_empty() { resolve_participants(participants); }
}
