use activitypub::controller::actor_exists;
use regex::Regex;
use url::Url;
use web_handler;

// *Notes*
//
// [TODO]
// Verification of HTTP signatures, therefore a validation of IDs is not needed
pub fn validate_activity(activity: serde_json::Value) -> Result<serde_json::Value, &'static str> {
    let known_type = if activity.get("type").is_some() {
        match activity["type"].as_str() {
            Some("Create") => true,
            Some("Update") => true,
            Some("Delete") => true,
            Some("Follow") => true,
            Some("Undo") => true,
            Some("Like") => true,
            Some("Announce") => true,
            _ => false,
        }
    } else {
        false
    };

    let valid_actor = if activity.get("actor").is_some() {
        if actor_exists(activity["actor"].as_str().unwrap()) {
            true
        } else {
            match web_handler::fetch_remote_object(activity["actor"].as_str().unwrap()) {
                Ok(remote_object) => {
                    let json_object: serde_json::Value =
                        serde_json::from_str(&remote_object).unwrap();
                    validate_actor(json_object).is_ok()
                }
                Err(_) => false,
            }
        }
    } else {
        false
    };

    if known_type && valid_actor {
        Ok(activity)
    } else {
        Err("Activity could not be validated")
    }
}

pub fn validate_object(object: serde_json::Value) -> Result<serde_json::Value, &'static str> {
    let known_type = if object.get("type").is_some() {
        match object["type"].as_str() {
            Some("Note") => true,
            Some("Article") => true,
            _ => false,
        }
    } else {
        false
    };

    // *Notes*
    //
    // [TODO]
    // Verification of IDs should be disabled if HTTP signatures are valid
    // The current behaviour marks non-public objects as invalid
    let valid_id = if object.get("id").is_some() {
        match parse_url(object["id"].as_str().unwrap()) {
            Ok(url) => valid_self_reference(&object, &url),
            Err(_) => false,
        }
    } else {
        false
    };

    let valid_actor = if object.get("attributedTo").is_some() {
        if actor_exists(object["attributedTo"].as_str().unwrap()) {
            true
        } else {
            match web_handler::fetch_remote_object(object["attributedTo"].as_str().unwrap()) {
                Ok(remote_object) => {
                    let json_object: serde_json::Value =
                        serde_json::from_str(&remote_object).unwrap();
                    validate_actor(json_object).is_ok()
                }
                Err(_) => false,
            }
        }
    } else {
        false
    };

    if known_type && valid_id && valid_actor {
        Ok(object)
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
    match web_handler::fetch_remote_object(url) {
        Ok(remote_object) => {
            let json_object: serde_json::Value = serde_json::from_str(&remote_object).unwrap();

            &json_object == object
        }
        Err(_) => false,
    }
}
