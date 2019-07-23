use database::PooledConnection;
use kibou_api;
use rocket_contrib::json::JsonValue;

#[get("/api/kibou/activities")]
pub fn activities(pooled_connection: PooledConnection) -> JsonValue {
    return kibou_api::public_activities(&pooled_connection);
}
