use activity;
use database;
use env;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use serde::{Serialize, Deserialize};
use serde_json;

// ActivityStreams2/AcitivityPub properties are expressed in CamelCase

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Activity
{
    // Properties according to
    // - https://www.w3.org/TR/activitystreams-core/#activities

    #[serde(rename = "@context")]
    pub context: Vec<String>,
    #[serde(rename = "type")]
    pub _type: String,
    pub id: String,
    pub actor: String,
    pub object: serde_json::Value,
    pub published: String,
    pub to: Vec<String>,
    pub cc: Vec<String>
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Object
{
    #[serde(rename = "type")]
    pub _type: String,
    pub id: String,
    pub published: String,
    pub attributedTo: String,
    pub inReplyTo: Option<String>,
    pub summary: Option<String>,
    pub content: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub tag: Vec<serde_json::Value>
}

#[derive(Serialize, Deserialize)]
pub struct Tag
{
    #[serde(rename = "type")]
    pub _type: String,
    pub href: String,
    pub name: String
}

pub fn get_activity_json_by_id(id: String) -> JsonValue
{
    let database = database::establish_connection();
    let activity_id = format!("{}://{}/activities/{}", env::get_value(String::from("endpoint.base_scheme")), env::get_value(String::from("endpoint.base_domain")), id);

    match activity::get_ap_activity_by_id(&database, &activity_id)
    {
        Ok(activity) => json!(serialize_from_internal_activity(activity).object),
        Err(_) => json!({"error": "Object not found."})
    }
}

pub fn get_object_json_by_id(id: String) -> JsonValue
{
    let database = database::establish_connection();
    let object_id = format!("{}://{}/objects/{}", env::get_value(String::from("endpoint.base_scheme")), env::get_value(String::from("endpoint.base_domain")), id);

    match activity::get_ap_object_by_id(&database, &object_id)
    {
        Ok(activity) => json!(serialize_from_internal_activity(activity).object),
        Err(_) => json!({"error": "Object not found."})
    }
}

pub fn serialize_from_internal_activity(activity: activity::Activity) -> Activity
{
    let payload: Activity = serde_json::from_value(activity.data).unwrap();

    payload
}

pub fn create_internal_activity(json_activity: serde_json::Value, actor_uri: String) -> activity::Activity
{
    activity::Activity
    {
        id: 0,
        data: json_activity,
        actor: actor_uri
    }
}
