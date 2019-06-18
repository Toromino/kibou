use kibou_api;
use rocket_contrib::json::JsonValue;

#[get("/api/kibou/activities")]
pub fn activities() -> JsonValue {
    return kibou_api::route_activities();
}
