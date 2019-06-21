use actor;
use chrono::Utc;
use database;

pub fn create_local_test_actor(username: &str) -> actor::Actor {
    let database = database::establish_connection();
    let mut test_actor = actor::Actor {
        id: 0,
        email: Some(format!("{}@example.tld", &username)),
        password: Some(String::from("Cy72MZfbfvDk7vnj")),
        actor_uri: format!("https://example.tld/{}", &username),
        username: Some(String::from("Alyssa P. Hacker")),
        preferred_username: String::from(username),
        summary: Some(String::from("Hey it's me, Alyssa!")),
        inbox: None,
        icon: Some(String::from("https://i.imgur.com/NXOJzr3.png")),
        keys: serde_json::json!({}),
        local: true,
        followers: serde_json::json!({"activitypub": []}),
        created: Utc::now().naive_utc(),
        modified: Utc::now().naive_utc(),
    };

    actor::create_actor(&database, &mut test_actor);
    test_actor.id = actor::get_actor_by_uri(&database, &test_actor.actor_uri)
        .unwrap()
        .id;
    test_actor
}

pub fn delete_test_actor(mut actor: actor::Actor) {
    let database = database::establish_connection();
    actor::delete(&database, actor);
}

pub fn create_remote_test_actor(username: &str) -> actor::Actor {
    let database = database::establish_connection();
    let mut test_actor = actor::Actor {
        id: 0,
        email: None,
        password: None,
        actor_uri: format!("https://remote.tld/{}", username),
        username: Some(String::from("Ben Bitdiddle")),
        preferred_username: String::from(username),
        summary: Some(String::from("A hardware expert")),
        inbox: Some(String::from("https://remote.tld/inbox")),
        icon: Some(String::from("https://i.imgur.com/NXOJzr3.png")),
        keys: serde_json::json!({}),
        local: false,
        followers: serde_json::json!({"activitypub": []}),
        created: Utc::now().naive_utc(),
        modified: Utc::now().naive_utc(),
    };

    actor::create_actor(&database, &mut test_actor);
    test_actor.id = actor::get_actor_by_uri(&database, &test_actor.actor_uri)
        .unwrap()
        .id;
    test_actor
}

pub fn valid_local_dummy_create_activity(
    object_id: String,
    in_reply_to: Option<String>,
) -> serde_json::Value {
    serde_json::json!({
        "context": ["https://www.w3.org/ns/activitystreams", "https://w3id.org/security/v1"],
        "type": "Create",
        "id": "https://example.tld/activities/82ea9f28-ae53-4cbf-925e-5e5c37fd12f1",
        "actor": "https://example.tld/alyssa",
        "object": {
            "type": "Note",
            "id": object_id,
            "attributedTo": "https://example.tld/alyssa",
            "inReplyTo": in_reply_to,
            "content": "Hello!",
            "published": "2015-02-10T15:04:55Z",
            "to": ["https://remote.tld/ben"],
            "cc": ["https://www.w3.org/ns/activitystreams#Public"]
        },
        "published": "2015-02-10T15:04:55Z",
        "to": ["https://remote.tld/ben"],
        "cc": ["https://www.w3.org/ns/activitystreams#Public"]
    })
}

pub fn valid_remote_dummy_create_activity(
    object_id: String,
    in_reply_to: Option<String>,
) -> serde_json::Value {
    serde_json::json!({
        "context": ["https://www.w3.org/ns/activitystreams", "https://w3id.org/security/v1"],
        "type": "Create",
        "id": "https://remote.tld/activities/10202",
        "actor": "https://remote.tld/ben",
        "object": {
            "type": "Note",
            "id": object_id,
            "attributedTo": "https://remote.tld/ben",
            "inReplyTo": in_reply_to,
            "content": "Listening to Pink Floyd right now!",
            "published": "2015-02-10T15:04:55Z",
            "to": ["https://example.tld/alyssa"],
            "cc": ["https://remote.tld/ben/followers", "https://www.w3.org/ns/activitystreams#Public"]
        },
        "published": "2015-02-10T15:04:55Z",
        "to": ["https://example.tld/alyssa"],
        "cc": ["https://remote.tld/ben/followers", "https://www.w3.org/ns/activitystreams#Public"]
    })
}
