use activity;
use database;
use env;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};

// ActivityStreams2/AcitivityPub properties are expressed in CamelCase

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
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

pub fn get_activity_json_by_id(id: &str) -> serde_json::Value {
    let database = database::establish_connection();
    let activity_id = format!(
        "{}://{}/activities/{}",
        env::get_value(String::from("endpoint.base_scheme")),
        env::get_value(String::from("endpoint.base_domain")),
        id
    );

    match activity::get_ap_activity_by_id(&database, &activity_id) {
        Ok(activity) => json!(serialize_from_internal_activity(activity).object),
        Err(_) => json!({"error": "Object not found."}),
    }
}

pub fn get_object_json_by_id(id: &str) -> serde_json::Value {
    let database = database::establish_connection();
    let object_id = format!(
        "{}://{}/objects/{}",
        env::get_value(String::from("endpoint.base_scheme")),
        env::get_value(String::from("endpoint.base_domain")),
        id
    );

    match activity::get_ap_object_by_id(&database, &object_id) {
        Ok(activity) => json!(serialize_from_internal_activity(activity).object),
        Err(_) => json!({"error": "Object not found."}),
    }
}

pub fn serialize_from_internal_activity(activity: activity::Activity) -> Activity {
    return serde_json::from_value(activity.data).unwrap();
}

pub fn create_internal_activity(
    json_activity: &serde_json::Value,
    actor_uri: &str,
) -> activity::Activity {
    activity::Activity {
        id: 0,
        data: json_activity.clone().to_owned(),
        actor: actor_uri.to_string(),
    }
}
