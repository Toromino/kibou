use actor::get_actor_by_acct;
use database;
use rocket::http::RawStr;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;

#[get("/.well-known/webfinger?<resource>")]
pub fn webfinger(resource: &RawStr) -> JsonValue
{
    let database = database::establish_connection();

    match get_actor_by_acct(&database, str::replace(resource.as_str(), "acct:", ""))
    {
        Ok(actor) =>
        {
            if actor.local
            {
                json!({
                    "subject": resource.as_str(),

                    "links": [
                    {
                        "rel": "self",
                        "type": "application/activity+json",
                        "href": actor.actor_uri
                    }
                    ]
                })
            }
            else { json!({"error": "User not found."}) }
        },
        Err(_) => json!({"error": "User not found."})
    }
}
