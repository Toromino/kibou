use activitypub::activity as ap_activity;
use activitypub::actor as ap_actor;
use activitypub::controller;
use activitypub::ActivitypubMediatype;
use activitypub::ActivitystreamsResponse;
use activitypub::HTTPSignature;
use rocket::Data;
use serde_json;
use std::io::Read;

#[get("/activities/<id>")]
pub fn activity(media_type: ActivitypubMediatype, id: String) -> ActivitystreamsResponse {
    ActivitystreamsResponse(ap_activity::get_activity_json_by_id(&id).to_string())
}

#[get("/actors/<handle>")]
pub fn actor(media_type: ActivitypubMediatype, handle: String) -> ActivitystreamsResponse {
    ActivitystreamsResponse(ap_actor::get_json_by_preferred_username(&handle).to_string())
}

#[post("/actors/<id>/inbox", data = "<activity>")]
pub fn actor_inbox(id: String, activity: Data, _signature: HTTPSignature) {
    let mut data_buffer = Vec::new();
    activity.open().read(&mut data_buffer);
    controller::prepare_incoming(
        serde_json::from_str(&String::from_utf8(data_buffer).unwrap())
            .unwrap_or_else(|_| serde_json::json!({})),
        _signature,
    );
}

#[post("/inbox", data = "<activity>")]
pub fn inbox(activity: Data, _signature: HTTPSignature) {
    let mut data_buffer = Vec::new();
    activity.open().read(&mut data_buffer);
    controller::prepare_incoming(
        serde_json::from_str(&String::from_utf8(data_buffer).unwrap())
            .unwrap_or_else(|_| serde_json::json!({})),
        _signature,
    );
}

#[get("/objects/<id>")]
pub fn object(media_type: ActivitypubMediatype, id: String) -> ActivitystreamsResponse {
    ActivitystreamsResponse(ap_activity::get_object_json_by_id(&id).to_string())
}
