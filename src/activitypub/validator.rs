pub fn validate_activity(activity: serde_json::Value) -> Result<serde_json::Value, &'static str>
{
    let known_type = match activity["type"].as_str() {
        Some("Create") => true,
        Some("Update") => true,
        Some("Delete") => true,
        Some("Follow") => true,
        Some("Unfollow") => true,
        Some("Like") => true,
        Some("Announce") => true,
        _ => false
    };

    if known_type { Ok(activity) } else { Err("Activity could not be validated") }
}

pub fn validate_object(object: serde_json::Value) -> Result<serde_json::Value, &'static str>
{
    let known_type = match object["type"].as_str() {
        Some("Note") => true,
        Some("Article") => true,
        _ => false
    };

    if known_type { Ok(object) } else { Err("Object could not be validated") }
}

pub fn validate_actor(actor: serde_json::Value) -> Result<serde_json::Value, &'static str>
{
    let known_type = match actor["type"].as_str() {
        Some("Person") => true,
        _ => false
    };

    if known_type { Ok(actor) } else { Err("Object could not be validated") }
}
