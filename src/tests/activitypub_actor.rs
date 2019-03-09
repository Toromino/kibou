use actor as internal_actor;
use activitypub::actor as actor;
use tests::utils::create_local_test_actor;
use tests::utils::delete_test_actor;

#[test]
fn get_err_by_preferred_username()
{
    let json_object = actor::get_json_by_preferred_username(String::from("希"));
    assert_eq!(json_object["error"].to_string(), format!("\"{}\"", "User not found."));
}

#[test]
fn get_json_by_preferred_username()
{
    let test_actor: internal_actor::Actor = create_local_test_actor("ap_actor_test_json");

    let email = test_actor.email.clone();
    let password = test_actor.password.clone();
    let actor_uri = test_actor.actor_uri.clone();
    let username = test_actor.username.clone().unwrap_or_else(|| test_actor.preferred_username.clone());
    let preferred_username = test_actor.preferred_username.clone();
    let summary = test_actor.summary.clone().unwrap_or_else(|| String::from(""));
    let local = test_actor.local.clone();

    let json_object = actor::get_json_by_preferred_username(preferred_username.clone());

    delete_test_actor(test_actor);
    assert_eq!(json_object["id"].to_string(), format!("\"{}\"", actor_uri.clone()));
    assert_eq!(json_object["summary"].to_string(), format!("\"{}\"", summary));
    assert_eq!(json_object["following"].to_string(), format!("\"{}/following\"", actor_uri.clone()));
    assert_eq!(json_object["followers"].to_string(), format!("\"{}/followers\"", actor_uri.clone()));
    assert_eq!(json_object["inbox"].to_string(), format!("\"{}/inbox\"", actor_uri.clone()));
    assert_eq!(json_object["outbox"].to_string(), format!("\"{}/outbox\"", actor_uri.clone()));
    assert_eq!(json_object["preferredUsername"].to_string(), format!("\"{}\"", preferred_username));
    assert_eq!(json_object["name"].to_string(), format!("\"{}\"", username));
    assert_eq!(json_object["url"].to_string(), format!("\"{}\"", actor_uri));
}
