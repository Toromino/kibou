use activitypub::activity::Activity;
use actor;
use chrono::Utc;
use database;
use env;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};

// ActivityStreams2/AcitivityPub properties are expressed in CamelCase
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Actor {
    // Properties according to
    // - https://www.w3.org/TR/activitypub/#actor-objects
    // - https://www.w3.org/TR/activitystreams-core/#actors
    #[serde(rename = "@context")]
    pub context: Option<Vec<serde_json::Value>>,
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
    pub icon: Option<serde_json::Value>,
    pub endpoints: serde_json::Value,
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
        actor::update_followers(&database, &mut actor);
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
        actor::update_followers(&database, &mut actor);
    }
}

// Refetches remote actor and detects changes to icon, username, keys and summary
// [TODO]
pub fn refresh() {}

pub fn get_json_by_preferred_username(preferred_username: String) -> serde_json::Value {
    let database = database::establish_connection();

    match actor::get_local_actor_by_preferred_username(&database, preferred_username) {
        Ok(actor) => json!(serialize_from_internal_actor(&actor)),
        Err(_) => json!({"error": "User not found."}),
    }
}

pub fn serialize_from_internal_actor(actor: &actor::Actor) -> Actor {
    Actor {
        context: Some(vec![
            serde_json::json!("https://www.w3.org/ns/activitystreams"),
            serde_json::json!("https://w3id.org/security/v1"),
        ]),
        _type: String::from("Person"),
        id: actor.actor_uri.clone(),
        summary: actor.summary.clone(),
        following: format!("{}/following", actor.actor_uri.clone()),
        followers: format!("{}/followers", actor.actor_uri.clone()),
        inbox: format!("{}/inbox", actor.actor_uri.clone()),
        outbox: format!("{}/outbox", actor.actor_uri.clone()),
        preferredUsername: actor.preferred_username.clone(),
        name: actor.username.clone(),
        publicKey: serde_json::json!({
            "id": format!("{}#main-key", actor.actor_uri.clone()),
            "owner": actor.actor_uri.clone(),
            "publicKeyPem": actor.keys["public"].clone()
        }),
        url: actor.actor_uri.clone(),
        icon: Some(serde_json::json!({"url":actor.icon,
        "type": "Image"})),
        endpoints: serde_json::json!({
            "sharedInbox":
                format!(
                    "{}://{}/inbox",
                    env::get_value(String::from("endpoint.base_scheme")),
                    env::get_value(String::from("endpoint.base_domain"))
                )
        }),
    }
}

pub fn create_internal_actor(ap_actor: Actor) -> actor::Actor {
    let actor_inbox = if ap_actor.endpoints.get("sharedInbox").is_some() {
        Some(
            ap_actor.endpoints["sharedInbox"]
                .as_str()
                .unwrap()
                .to_string(),
        )
    } else {
        Some(ap_actor.inbox)
    };

    let actor_icon = if ap_actor.icon.is_some() {
        Some(ap_actor.icon.unwrap()["url"].as_str().unwrap().to_string())
    } else {
        None
    };

    actor::Actor {
        id: 0, // Fill with placeholder value, this property will get ignored
        email: None,
        password: None,
        actor_uri: ap_actor.id,
        username: ap_actor.name,
        preferred_username: ap_actor.preferredUsername,
        summary: ap_actor.summary,
        inbox: actor_inbox,
        icon: actor_icon,
        local: false,
        keys: serde_json::json!({"public" : ap_actor.publicKey["publicKeyPem"]}),
        followers: serde_json::json!({"activitypub": []}),
        created: Utc::now().naive_utc(),
    }
}
