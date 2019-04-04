use activitypub::activity as ap_activity;
use activitypub::actor as ap_actor;
use activitypub::controller;
use activitypub::HTTPSignature;
use rocket_contrib::json::JsonValue;
use serde_json;

#[get("/activities/<id>", format = "application/activity+json")]
pub fn activity(id: String) -> JsonValue {
    ap_activity::get_activity_json_by_id(id)
}

#[get("/actors/<handle>", format = "application/activity+json")]
pub fn actor(handle: String) -> JsonValue {
    ap_actor::get_json_by_preferred_username(handle)
}

#[post("/actors/<id>/inbox", data = "<activity>")]
pub fn actor_inbox(id: String, activity: String, _signature: HTTPSignature) {
    controller::prepare_incoming(
        serde_json::from_str(&activity).unwrap_or_else(|_| serde_json::json!({})),
        _signature,
    );
}

#[post("/inbox", data = "<activity>")]
pub fn inbox(activity: String, _signature: HTTPSignature) {
    controller::prepare_incoming(
        serde_json::from_str(&activity).unwrap_or_else(|_| serde_json::json!({})),
        _signature,
    );
}

#[get("/objects/<id>", format = "application/activity+json")]
pub fn object(id: String) -> JsonValue {
    ap_activity::get_object_json_by_id(id)
}
