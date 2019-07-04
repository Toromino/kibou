use activitypub::activity as ap_activity;
use activitypub::actor as ap_actor;
use activitypub::controller;
use activitypub::ActivitypubMediatype;
use activitypub::ActivitystreamsResponse;
use activitypub::Signature;
use rocket::http::Status;
use serde_json;

#[get("/activities/<id>")]
pub fn activity(_media_type: ActivitypubMediatype, id: String) -> ActivitystreamsResponse {
    ActivitystreamsResponse(ap_activity::get_activity_json_by_id(&id).to_string())
}

#[get("/actors/<handle>")]
pub fn actor(_media_type: ActivitypubMediatype, handle: String) -> ActivitystreamsResponse {
    ActivitystreamsResponse(ap_actor::get_json_by_preferred_username(&handle).to_string())
}

#[post("/actors/<id>/inbox", data = "<activity>")]
pub fn actor_inbox(id: String, activity: String, _signature: Signature) -> Status {
    match serde_json::from_str(&activity) {
        Ok(serialized_activity) => {
            controller::prepare_incoming(serialized_activity, _signature);
            return rocket::http::Status::Ok;
        }
        Err(_) => return rocket::http::Status::BadRequest,
    }
}

#[post("/inbox", data = "<activity>")]
pub fn inbox(activity: String, _signature: Signature) -> Status {
    match serde_json::from_str(&activity) {
        Ok(serialized_activity) => {
            controller::prepare_incoming(serialized_activity, _signature);
            return rocket::http::Status::Ok;
        }
        Err(_) => return rocket::http::Status::BadRequest,
    }
}

#[get("/objects/<id>")]
pub fn object(_media_type: ActivitypubMediatype, id: String) -> ActivitystreamsResponse {
    ActivitystreamsResponse(ap_activity::get_object_json_by_id(&id).to_string())
}
