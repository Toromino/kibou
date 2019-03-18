use actor;
use database;
use kibou_api::account;
use tests::utils::create_local_test_actor;
use tests::utils::delete_test_actor;

#[test]
fn follow() {
    let database = database::establish_connection();

    let test_actor = create_local_test_actor("b701052f-9718-4030-8d6a-c97eeee136ef");
    let test_actor_uri = test_actor.actor_uri.clone();

    let test_follower_1 = create_local_test_actor("d530d627-ad50-4780-bae4-7e475d924970");
    let test_follower_1_uri = test_follower_1.actor_uri.clone();

    account::follow(test_follower_1_uri.clone(), test_actor_uri);

    let serialized_test_actor = actor::get_actor_by_uri(&database, &test_actor.actor_uri).unwrap();
    let activitypub_followers: serde_json::Value =
        serialized_test_actor.followers["activitypub"].clone();
    let test_actor_follow_data: Vec<serde_json::Value> =
        serde_json::from_value(activitypub_followers).unwrap_or_else(|_| vec![]);

    delete_test_actor(test_actor);
    delete_test_actor(test_follower_1);

    assert_eq!(test_actor_follow_data[0]["href"], test_follower_1_uri);
}
