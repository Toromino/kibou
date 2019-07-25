use activitypub;
use activitypub::controller::actor_exists;
use activitypub::controller::fetch;
use activitypub::controller::object_exists;
use activitypub::Activity;
use activitypub::Object;
use actor;
use database::PooledConnection;
use html;
use regex::Regex;
use url::Url;
use web;
use web::http_signatures::Signature;

pub fn validate_activity(
    pooled_connection: &PooledConnection,
    mut activity: serde_json::Value,
    signature: Signature,
) -> Result<serde_json::Value, &'static str> {
    let known_type = if activity.get("type").is_some() {
        match activity["type"].as_str() {
            Some("Accept") => true,
            Some("Announce") => true,
            Some("Create") => true,
            Some("Follow") => true,
            Some("Like") => true,
            Some("Undo") => true,
            _ => false,
        }
    } else {
        false
    };

    let valid_actor = if activity.get("actor").is_some() {
        if actor_exists(pooled_connection, activity["actor"].as_str().unwrap()) {
            activitypub::actor::refresh(activity["actor"].as_str().unwrap().to_string());
            true
        } else {
            fetch(pooled_connection, activity["actor"].as_str().unwrap());
            actor_exists(pooled_connection, activity["actor"].as_str().unwrap())
        }
    } else {
        false
    };

    let valid_signature = signature.verify(
        &mut actor::get_actor_by_uri(pooled_connection, activity["actor"].as_str().unwrap()).unwrap(),
    );

    let valid_object = if activity["type"].as_str() == Some("Create") {
        if !object_exists(activity["object"]["id"].as_str().unwrap()) {
            match validate_object(pooled_connection, activity["object"].clone(), valid_signature) {
                Ok(object) => {
                    activity["object"] = object;
                    true
                }
                Err(_) => false,
            }
        } else {
            false
        }
    } else {
        true
    };

    if known_type && valid_actor && valid_signature && valid_object {
        Ok(normalize_activity(activity))
    } else {
        Err("Activity could not be validated")
    }
}

pub fn validate_object(
    pooled_connection: &PooledConnection,
    object: serde_json::Value,
    valid_signature: bool,
) -> Result<serde_json::Value, &'static str> {
    let known_type = if object.get("type").is_some() {
        match object["type"].as_str() {
            Some("Note") => true,
            Some("Article") => true,
            _ => false,
        }
    } else {
        false
    };

    let valid_id = if valid_signature {
        true
    } else {
        if object.get("id").is_some() {
            println!("Signature of object '{}' is invalid or missing, trying to verify it's source ...", object["id"].to_string());
            match parse_url(object["id"].as_str().unwrap()) {
                Ok(url) => valid_self_reference(&object, &url),
                Err(_) => false,
            }
        } else {
            false
        }
    };

    let valid_actor = if object.get("attributedTo").is_some() {
        if actor_exists(pooled_connection, object["attributedTo"].as_str().unwrap()) {
            activitypub::actor::refresh(object["attributedTo"].as_str().unwrap().to_string());
            true
        } else {
            fetch(pooled_connection, object["attributedTo"].as_str().unwrap());
            actor_exists(pooled_connection, object["attributedTo"].as_str().unwrap())
        }
    } else {
        false
    };

    if known_type && valid_id && valid_actor {
        Ok(normalize_object(object))
    } else {
        Err("Object could not be validated")
    }
}

pub fn validate_actor(actor: serde_json::Value) -> Result<serde_json::Value, &'static str> {
    let known_type = if actor.get("type").is_some() {
        match actor["type"].as_str() {
            Some("Person") => true,
            _ => false,
        }
    } else {
        false
    };

    let valid_id = if actor.get("id").is_some() {
        match parse_url(actor["id"].as_str().unwrap()) {
            Ok(url) => valid_self_reference(&actor, &url),
            Err(_) => false,
        }
    } else {
        false
    };

    let valid_preferred_username = if actor.get("preferredUsername").is_some() {
        let username_regex = Regex::new(r"^[A-Za-z0-9_]{1,32}$").unwrap();
        username_regex.is_match(actor["preferredUsername"].as_str().unwrap())
    } else {
        false
    };

    let valid_inbox = if actor.get("inbox").is_some() {
        match parse_url(actor["inbox"].as_str().unwrap()) {
            Ok(_) => true,
            Err(_) => false,
        }
    } else {
        false
    };

    let valid_public_key = if actor.get("publicKey").is_some() {
        match pem::parse(actor["publicKey"]["publicKeyPem"].as_str().unwrap()) {
            Ok(_) => true,
            Err(_) => false,
        }
    } else {
        false
    };

    if known_type && valid_id && valid_preferred_username && valid_inbox && valid_public_key {
        Ok(actor)
    } else {
        Err("Object could not be validated")
    }
}

fn normalize_activity(mut activity: serde_json::Value) -> serde_json::Value {
    if activity["to"].is_string() {
        activity["to"] = serde_json::json!(vec![activity["to"].as_str().unwrap().to_string()]);
    }

    if activity["cc"].is_string() {
        activity["cc"] = serde_json::json!(vec![activity["cc"].as_str().unwrap().to_string()]);
    }

    let mut new_activity: Activity;

    if activity.get("cc").is_none() {
        let new_cc_tag: Vec<String> = vec![];
        activity["cc"] = serde_json::json!(new_cc_tag);
    }

    new_activity = serde_json::from_value(activity.clone()).unwrap();
    new_activity.context = None;
    new_activity.to = normalize_public_addressing(new_activity.to);
    new_activity.cc = normalize_public_addressing(new_activity.cc);

    serde_json::to_value(new_activity).unwrap()
}

fn normalize_object(mut object: serde_json::Value) -> serde_json::Value {
    if object["to"].is_string() {
        object["to"] = serde_json::json!(vec![object["to"].as_str().unwrap().to_string()]);
    }

    if object["cc"].is_string() {
        object["cc"] = serde_json::json!(vec![object["cc"].as_str().unwrap().to_string()]);
    }

    let mut new_object: Object;

    if object.get("cc").is_none() {
        let new_cc_tag: Vec<String> = vec![];
        object["cc"] = serde_json::json!(new_cc_tag);
    }

    new_object = serde_json::from_value(object.clone()).unwrap();
    new_object.content = html::strip_tags(&new_object.content);
    new_object.context = None;
    new_object.to = normalize_public_addressing(new_object.to);
    new_object.cc = normalize_public_addressing(new_object.cc);

    serde_json::to_value(new_object).unwrap()
}

fn normalize_public_addressing(mut collection: Vec<String>) -> Vec<String> {
    let alternative_public_address = vec![
        "https://www.w3.org/ns/activitystreams",
        "Public",
        "as:Public",
    ];

    for address in alternative_public_address {
        if collection.contains(&address.to_string()) {
            let index = collection
                .iter()
                .position(|receipient| receipient == &address.to_string())
                .unwrap();
            collection[index] = String::from("https://www.w3.org/ns/activitystreams#Public");
        }
    }

    return collection;
}

fn parse_url(url: &str) -> Result<String, url::ParseError> {
    let mut parsed_url = String::new();
    let stripped_characters = "\"";
    for character in url.chars() {
        if !stripped_characters.contains(character) {
            parsed_url.push(character);
        }
    }

    match Url::parse(&parsed_url) {
        Ok(remote_url) => Ok(remote_url.to_string()),
        Err(e) => Err(e),
    }
}

fn valid_self_reference(object: &serde_json::Value, url: &str) -> bool {
    match web::fetch_remote_object(url) {
        Ok(remote_object) => {
            let json_object: serde_json::Value = serde_json::from_str(&remote_object).unwrap();

            &json_object == object
        }
        Err(_) => false,
    }
}
