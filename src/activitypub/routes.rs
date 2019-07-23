use activitypub::activity as ap_activity;
use activitypub::actor as ap_actor;
use activitypub::controller;
use activitypub::ActivitypubMediatype;
use activitypub::ActivitystreamsResponse;
use activitypub::Payload;
use activitypub::Signature;

#[get("/activities/<id>")]
pub fn activity(_media_type: ActivitypubMediatype, id: String) -> ActivitystreamsResponse {
    ActivitystreamsResponse(ap_activity::get_activity_json_by_id(&id).to_string())
}

#[get("/actors/<handle>")]
pub fn actor(_media_type: ActivitypubMediatype, handle: String) -> ActivitystreamsResponse {
    ActivitystreamsResponse(ap_actor::get_json_by_preferred_username(&handle).to_string())
}

#[post("/actors/<_id>/inbox", data = "<activity>")]
pub fn actor_inbox(_id: String, activity: Payload, _signature: Signature) {
    controller::prepare_incoming(activity.0, _signature);
}

#[post("/inbox", data = "<activity>")]
pub fn inbox(activity: Payload, _signature: Signature) {
    controller::prepare_incoming(activity.0, _signature);
}

#[get("/objects/<id>")]
pub fn object(_media_type: ActivitypubMediatype, id: String) -> ActivitystreamsResponse {
    ActivitystreamsResponse(ap_activity::get_object_json_by_id(&id).to_string())
}
