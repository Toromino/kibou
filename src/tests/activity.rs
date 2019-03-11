use activity;
use activitypub::activity as ap_activity;
use database;
use tests::utils::valid_local_dummy_create_activity;
use tests::utils::valid_remote_dummy_create_activity;

#[test]
fn get_ap_activity_by_id()
{
    let database = database::establish_connection();
    let test_activity = serde_json::json!({
        "context": ["https://www.w3.org/ns/activitystreams", "https://w3id.org/security/v1"],
        "type": "Create",
        "id": "https://example.tld/activities/5b4f7e70-8467-44ef-8278-9729b5c11ba7",
        "actor": "https://example.tld/alyssa",
        "object": {
            "type": "Note",
            "id": "https://example.tld/objects/b2192a75-8c9d-4fe0-9012-0b21e98e6e2b",
            "attributedTo": "https://example.tld/alyssa",
            "inReplyTo": null,
            "content": "Have you already received your swimsuit?",
            "published": "2015-02-10T15:04:55Z",
            "to": ["https://remote.tld/ben"],
            "cc": ["https://www.w3.org/ns/activitystreams#Public"]
        },
        "published": "2015-02-10T15:04:55Z",
        "to": ["https://remote.tld/ben"],
        "cc": ["https://www.w3.org/ns/activitystreams#Public"]
    });

    activity::insert_activity(&database, ap_activity::create_internal_activity(test_activity, String::from("https://example.tld/alyssa")));
    let result = activity::get_ap_activity_by_id(&database, "https://example.tld/activities/5b4f7e70-8467-44ef-8278-9729b5c11ba7");
    activity::delete_ap_activity_by_id(&database, String::from("https://example.tld/activities/5b4f7e70-8467-44ef-8278-9729b5c11ba7"));

    match result
    {
        Ok(_) => assert!(true),
        Err(_) => assert!(false, "AP activity should exist")
    }
}

#[test]
fn get_ap_object_by_id()
{
    let database = database::establish_connection();
    let test_object_id = String::from("https://remote.tld/objects/9c247a1d-5aed-4a3f-922d-d88eaf9938a2");
    let test_actor = String::from("https://remote.tld/ben");

    activity::insert_activity(&database, ap_activity::create_internal_activity(valid_remote_dummy_create_activity(test_object_id.clone(), None), String::from("https://remote.tld/ben")));
    let result = activity::get_ap_object_by_id(&database, &test_object_id);
    activity::delete_ap_object_by_id(&database, test_object_id);

    match result
    {
        Ok(_) => assert!(true),
        Err(_) => assert!(false, "AP object should exist")
    }
}

#[test]
fn get_ap_object_replies_by_id()
{
    let database = database::establish_connection();
    let test_object_id = String::from("https://remote.tld/objects/6d5fbaa1-82b6-434d-b885-7865b07deae4");
    let test_reply_id = String::from("https://remote.tld/objects/b7435596-e01d-4474-bbd5-b033149bddbf");

    activity::insert_activity(&database, ap_activity::create_internal_activity(valid_local_dummy_create_activity(test_object_id.clone(), None), String::from("https://example.tld/alyssa")));
    activity::insert_activity(&database, ap_activity::create_internal_activity(valid_remote_dummy_create_activity(test_reply_id.clone(), Some(test_object_id.clone())), String::from("https://remote.tld/ben")));
    match activity::get_ap_object_replies_by_id(&database, &test_object_id.clone())
    {
        Ok(activities) =>
        {
            activity::delete_ap_object_by_id(&database, test_object_id);
            activity::delete_ap_object_by_id(&database, test_reply_id.clone());
            assert_eq!(test_reply_id, activities[0].data["object"]["id"].as_str().unwrap().to_string())
        },
        Err(_) =>
        {
            activity::delete_ap_object_by_id(&database, test_object_id);
            activity::delete_ap_object_by_id(&database, test_reply_id);
            assert!(false, "Reply should exist")
        }
    }
}

#[test]
fn count_ap_object_replies_by_id()
{
    let database = database::establish_connection();
    let test_object_id = String::from("https://remote.tld/objects/1c34ec09-a2dd-4d43-85b9-2c63cbbd1f54");
    let test_reply_id = String::from("https://remote.tld/objects/a0aa8551-f1a7-4b83-9493-9dadc378599b");

    activity::insert_activity(&database, ap_activity::create_internal_activity(valid_local_dummy_create_activity(test_object_id.clone(), None), String::from("https://example.tld/alyssa")));
    activity::insert_activity(&database, ap_activity::create_internal_activity(valid_remote_dummy_create_activity(test_reply_id.clone(), Some(test_object_id.clone())), String::from("https://remote.tld/ben")));

    let reply_num: usize = activity::count_ap_object_replies_by_id(&database, &test_object_id.clone()).unwrap_or_else(|_| 0);
    activity::delete_ap_object_by_id(&database, test_object_id);
    activity::delete_ap_object_by_id(&database, test_reply_id);

    assert_eq!(reply_num, 1);
}
