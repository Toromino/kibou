use activitypub::activity::Object;
use activitypub::validator;
use tests::utils::create_remote_test_actor;
use tests::utils::delete_test_actor;

#[test]
fn validate_object() {
    let test_actor = create_remote_test_actor("a50a530d-6981-4715-a746-95eacbcceb7f");
    match validator::validate_object(
        serde_json::to_value(valid_dummy_object(&test_actor.actor_uri)).unwrap(),
        true,
    ) {
        Ok(_) => assert!(true),
        Err(_) => assert!(false, "Valid object should pass the validator"),
    }

    match validator::validate_object(serde_json::to_value(invalid_dummy_object()).unwrap(), true) {
        Ok(_) => assert!(false, "Invalid object should not pass the validator"),
        Err(_) => assert!(true),
    }
    delete_test_actor(test_actor);
}

fn valid_dummy_object(actor_uri: &str) -> Object {
    Object {
        context: Some(serde_json::json!(vec![
            String::from("https://www.w3.org/ns/activitystreams"),
            String::from("https://w3id.org/security/v1"),
        ])),
        _type: String::from("Note"),
        id: String::from("https://example.tld/objects/afb1c173-2ecd-4250-9bca-5e90d4340e06"),
        published: String::from("2015-02-10T15:04:55Z"),
        attributedTo: String::from(actor_uri),
        inReplyTo: None,
        summary: None,
        content: String::from("Haha it's me Alyssa!"),
        to: vec![],
        cc: vec![
            String::from("https://example.tld/users/alyssa/followers"),
            String::from("https://www.w3.org/ns/activitystreams#Public"),
        ],
        tag: None,
    }
}

fn invalid_dummy_object() -> Object {
    Object {
        context: Some(serde_json::json!(vec![
            String::from("https://www.w3.org/ns/activitystreams"),
            String::from("https://w3id.org/security/v1"),
        ])),
        _type: String::from("Notice"),
        id: String::from("https://example.tld/objects/93254c3a-dd02-4987-adc7-abfb815799da"),
        published: String::from("2015-02-10T15:04:55Z"),
        attributedTo: String::from("https://example.tld/users/alyssa"),
        inReplyTo: None,
        summary: None,
        content: String::from("Listening to Pink Floyd right now"),
        to: vec![],
        cc: vec![
            String::from("https://example.tld/users/alyssa/followers"),
            String::from("https://www.w3.org/ns/activitystreams#Public"),
        ],
        tag: None,
    }
}
