use actor::get_actor_by_acct;
use database;
use rocket::http::RawStr;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;

#[get("/.well-known/webfinger?<resource>")]
pub fn webfinger(resource: &RawStr) -> JsonValue {
    let database = database::establish_connection();

    let mut parsed_resource: &str = &resource.as_str().replace("%3A", ":").replace("%40", "@");

    match get_actor_by_acct(&database, &str::replace(parsed_resource, "acct:", "")) {
        Ok(actor) => {
            if actor.local {
                json!({
                    "subject": parsed_resource,

                    "links": [
                    {
                        "rel": "self",
                        "type": "application/activity+json",
                        "href": actor.actor_uri
                    }
                    ]
                })
            } else {
                json!({"error": "User not found."})
            }
        }
        Err(_) => json!({"error": "User not found."}),
    }
}
