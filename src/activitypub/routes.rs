use activitypub::controller;
use activitypub::ActivitypubMediatype;
use activitypub::ActivitystreamsResponse;
use activitypub::Payload;
use activitypub::Signature;
use database::PooledConnection;
use rocket::http::Status;
use serde_json;

#[get("/activities/<id>")]
pub fn activity(
    pooled_connection: PooledConnection,
    _media_type: ActivitypubMediatype,
    id: String,
) -> ActivitystreamsResponse {
    return ActivitystreamsResponse(
        controller::activity_by_id(&pooled_connection, &id).to_string(),
    );
}

#[get("/actors/<handle>")]
pub fn actor(_media_type: ActivitypubMediatype, handle: String) -> ActivitystreamsResponse {
    return ActivitystreamsResponse(ap_actor::get_json_by_preferred_username(&handle).to_string());
}

#[post("/actors/<_id>/inbox", data = "<activity>")]
pub fn actor_inbox(pooled_connection: PooledConnection, _id: String, activity: Payload, _signature: Signature) -> Status {
    return inbox(pooled_connection, activity, _signature);
}

#[post("/inbox", data = "<activity>")]
pub fn inbox(pooled_connection: PooledConnection, activity: Payload, _signature: Signature) -> Status {
    return controller::validate_incoming(&pooled_connection, activity.0, _signature);
}

#[get("/objects/<id>")]
pub fn object(
    pooled_connection: PooledConnection,
    _media_type: ActivitypubMediatype,
    id: String,
) -> ActivitystreamsResponse {
    return ActivitystreamsResponse(controller::object_by_id(&pooled_connection, &id).to_string());
}
