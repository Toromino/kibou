use activity;
use activitypub::activity::create_internal_activity;
use activitypub::controller as controller;
use actor;
use database;
use tests::utils::create_local_test_actor;
use tests::utils::delete_test_actor;
use tests::utils::valid_remote_dummy_create_activity;

#[test]
fn err_actor_exists()
{
    let test_actor_uri = String::from("https://example.tld/actors/希");
    let test_actor_exists = controller::actor_exists(&test_actor_uri);

    assert_eq!(test_actor_exists, false);
}

#[test]
fn actor_exists()
{
    let test_actor: actor::Actor = create_local_test_actor("b9f4e4bc-828e-403a-bdf9-66b618ebac60");
    let test_actor_exists = controller::actor_exists(&test_actor.actor_uri);
    delete_test_actor(test_actor);

    assert_eq!(test_actor_exists, true);
}

#[test]
fn err_object_exists()
{
    let test_object_id = String::from("https://remote.tld/objects/bfdde7c2-9267-445c-bb0c-3196a8854284");
    let test_object_exists = controller::object_exists(&test_object_id);

    assert_eq!(test_object_exists, false);
}

#[test]
fn object_exists()
{
    let database = database::establish_connection();
    let test_object_id = String::from("https://remote.tld/objects/ad8127cd-bbf5-4910-bdca-bad648fa0901");
    let test_actor = String::from("https://remote.tld/ben");

    activity::insert_activity(&database, create_internal_activity(valid_remote_dummy_create_activity(test_object_id.clone(), None), test_actor));
    let test_object_exists = controller::object_exists(&test_object_id.clone());
    activity::delete_ap_object_by_id(&database, test_object_id);

    assert_eq!(test_object_exists, true);
}
