use activitypub::actor::add_follow;
use actor;
use base64;
use bcrypt::verify;
use chrono::Utc;
use database;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Verifier;
use serde_json;
use tests::utils::create_local_test_actor;
use tests::utils::create_remote_test_actor;
use tests::utils::delete_test_actor;

#[test]
fn generate_new_keys() {
    let mut test_actor = create_local_test_actor("852f47fe-9271-4f96-ae81-610ae40ca164");
    let test_actor_pkey = test_actor.get_private_key();
    delete_test_actor(test_actor);

    match pem::parse(test_actor_pkey) {
        Ok(_) => assert!(true),
        Err(_) => assert!(false, "Invalid private key was generated"),
    }
}

#[test]
fn sign() {
    let mut test_actor = create_local_test_actor("92edfc1f-4f9d-40f3-83a0-e4deb5163c15");
    let test_string: String = String::from("Test");
    let test_actor_pkey = test_actor.get_private_key();
    let signed_string = test_actor.sign(test_string.clone());
    delete_test_actor(test_actor);

    let pem_decoded = pem::parse(test_actor_pkey).unwrap();
    let pkey =
        PKey::from_rsa(openssl::rsa::Rsa::private_key_from_der(&pem_decoded.contents).unwrap())
            .unwrap();
    let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey).unwrap();
    verifier.update(&test_string.into_bytes()).unwrap();

    assert_eq!(
        verifier
            .verify(&base64::decode(&signed_string.into_bytes()).unwrap())
            .unwrap(),
        true
    );
}

#[test]
fn update_local_keys() {
    let mut test_actor = create_local_test_actor("d0fd62bc-a991-42e5-9f39-758bafab06e6");
    let test_actor_pkey = test_actor.get_private_key();
    test_actor.update_local_keys();
    let test_actor_new_pkey = test_actor.get_private_key();
    delete_test_actor(test_actor);

    assert_ne!(test_actor_pkey, test_actor_new_pkey);
}

#[test]
fn is_actor_followed_by() {
    let database = database::establish_connection();
    let test_actor = create_local_test_actor("d99eb450-65a0-4c9a-8092-b1b0fd71941a");
    let test_follower_1 = create_remote_test_actor("51df1b86-db1c-458b-9f76-7032275c867b");
    let test_follower_1_uri = test_follower_1.actor_uri.clone();

    add_follow(&test_actor.actor_uri, &test_follower_1_uri, "");

    match actor::is_actor_followed_by(&database, &test_actor, &test_follower_1.actor_uri) {
        Ok(true) => {
            delete_test_actor(test_actor);
            delete_test_actor(test_follower_1);

            assert!(true)
        }
        Ok(false) => {
            delete_test_actor(test_actor);
            delete_test_actor(test_follower_1);

            assert!(false, "Follow should exist")
        }
        Err(_) => {
            delete_test_actor(test_actor);
            delete_test_actor(test_follower_1);

            assert!(false, "An error occured")
        }
    }
}

#[test]
fn get_actor_by_acct() {
    let database = database::establish_connection();
    let mut test_actor = create_local_test_actor("6ca9fede-04e0-4175-b7af-29ff760b80ba");
    let test_actor_id = test_actor.id;
    let acct = test_actor.get_acct();

    match actor::get_actor_by_acct(&database, &acct) {
        Ok(actor) => {
            delete_test_actor(test_actor);
            assert_eq!(test_actor_id, actor.id);
        }
        Err(_) => {
            delete_test_actor(test_actor);
            assert!(false, "Actor could not be queried by `acct`");
        }
    }
}

#[test]
fn get_actor_by_uri() {
    let database = database::establish_connection();
    let test_actor = create_local_test_actor("11c116b1-818c-4c07-aac7-26debc450d0b");
    let test_actor_id = test_actor.id;

    match actor::get_actor_by_uri(&database, &test_actor.actor_uri) {
        Ok(actor) => {
            delete_test_actor(test_actor);
            assert_eq!(test_actor_id, actor.id);
        }
        Err(_) => {
            delete_test_actor(test_actor);
            assert!(false, "Actor could not be queried by `uri`");
        }
    }
}

#[test]
fn get_local_actor_by_preferred_username() {
    let database = database::establish_connection();
    let mut test_actor = create_local_test_actor("dd4f078d-d5d8-4f84-a99f-7c13841fa962");

    match actor::get_local_actor_by_preferred_username(&database, &test_actor.preferred_username) {
        Ok(_) => {
            delete_test_actor(test_actor);
            assert!(true);
        }
        Err(_) => {
            delete_test_actor(test_actor);
            assert!(false, "Local user was not found");
        }
    }
}

#[test]
fn create_remote_actor() {
    let database = database::establish_connection();
    let test_actor = create_remote_test_actor("6c85897a-d420-4e23-b5c8-7d6eb4ea834f");
    let email = test_actor.email.clone();
    let password = test_actor.password.clone();
    let actor_uri = test_actor.actor_uri.clone();
    let username = test_actor.username.clone();
    let preferred_username = test_actor.preferred_username.clone();
    let summary = test_actor.summary.clone();
    let local = test_actor.local.clone();

    match actor::get_actor_by_uri(&database, &test_actor.actor_uri) {
        Ok(db_actor) => {
            delete_test_actor(test_actor);
            assert_eq!(db_actor.email, email);
            assert_eq!(db_actor.password, password);
            assert_eq!(db_actor.actor_uri, actor_uri);
            assert_eq!(db_actor.username, username);
            assert_eq!(db_actor.preferred_username, preferred_username.clone());
            assert_eq!(db_actor.summary, summary);
            assert_eq!(db_actor.local, local);
        }
        Err(_) => {
            delete_test_actor(test_actor);
            assert!(false, "Actor was not found in database");
        }
    }
}

#[test]
fn delete_remote_actor() {
    let database = database::establish_connection();
    let test_actor = create_remote_test_actor("3e1218d3-f735-4a50-b610-52c6f64445ee");
    let actor_uri = test_actor.actor_uri.clone();
    delete_test_actor(test_actor);

    match actor::get_actor_by_uri(&database, &actor_uri) {
        Ok(_) => assert!(false, "Actor was not deleted!"),
        Err(_) => assert!(true),
    }
}

/// Creates an actor with all nullable values being filled with null
///
/// # Note
/// This is neither a valid local actor, nor a valid remote actor. It's purpose is to test
/// the creation of actors with optional values in general.
#[test]
fn create_actor_with_optional_values() {
    let database = database::establish_connection();
    let test_actor_username = String::from("e40e4712-ad48-4b5b-9bd6-1459b399b85c");

    let mut test_actor = actor::Actor {
        id: 0,
        email: None,
        password: None,
        actor_uri: String::from("https://example.tld/e40e4712-ad48-4b5b-9bd6-1459b399b85c"),
        username: None,
        preferred_username: String::from("e40e4712-ad48-4b5b-9bd6-1459b399b85c"),
        summary: None,
        inbox: None,
        icon: None,
        keys: serde_json::json!({}),
        local: false,
        followers: serde_json::json!({"activitypub": []}),
        created: Utc::now().naive_utc(),
        modified: Utc::now().naive_utc(),
    };

    let email = test_actor.email.clone();
    let password = test_actor.password.clone();
    let actor_uri = test_actor.actor_uri.clone();
    let username = test_actor.username.clone();
    let preferred_username = test_actor.preferred_username.clone();
    let summary = test_actor.summary.clone();
    let inbox = test_actor.inbox.clone();
    let icon = test_actor.icon.clone();
    let keys = test_actor.keys.clone();
    let local = test_actor.local.clone();

    actor::create_actor(&database, &mut test_actor);

    match actor::get_actor_by_uri(&database, &actor_uri) {
        Ok(db_actor) => {
            delete_test_actor(test_actor);
            assert_eq!(db_actor.email, email);
            assert_eq!(db_actor.password, password);
            assert_eq!(db_actor.actor_uri, actor_uri);
            assert_eq!(db_actor.username, username);
            assert_eq!(db_actor.preferred_username, preferred_username);
            assert_eq!(db_actor.summary, summary);
            assert_eq!(db_actor.inbox, inbox);
            assert_eq!(db_actor.icon, icon);
            assert_eq!(db_actor.keys, keys);
            assert_eq!(db_actor.local, local);
        }
        Err(_) => {
            delete_test_actor(test_actor);
            assert!(false, "Actor was not found in database");
        }
    }
}

#[test]
fn create_local_actor() {
    let database = database::establish_connection();
    let test_actor = create_local_test_actor("efdd9c02-2887-47a6-a2a7-97e8cb533a7c");

    let email = test_actor.email.clone();
    let password = test_actor.password.clone();
    let actor_uri = test_actor.actor_uri.clone();
    let username = test_actor.username.clone();
    let preferred_username = test_actor.preferred_username.clone();
    let summary = test_actor.summary.clone();
    let local = test_actor.local.clone();

    match actor::get_local_actor_by_preferred_username(&database, &test_actor.preferred_username) {
        Ok(db_actor) => {
            delete_test_actor(test_actor);
            assert_eq!(db_actor.email, email);
            assert_eq!(db_actor.password, password);
            assert_eq!(db_actor.actor_uri, actor_uri);
            assert_eq!(db_actor.username, username);
            assert_eq!(db_actor.preferred_username, preferred_username.clone());
            assert_eq!(db_actor.summary, summary);
            assert_eq!(db_actor.local, local);
            assert!(
                true,
                verify(
                    String::from("Cy72MZfbfvDk7vnj").into_bytes(),
                    &db_actor.password.unwrap()
                )
            );
        }
        Err(_) => {
            delete_test_actor(test_actor);
            assert!(false, "Actor was not found in database");
        }
    }
}

#[test]
fn delete_local_actor() {
    let database = database::establish_connection();
    let test_actor: actor::Actor = create_local_test_actor("e649852f-61aa-4999-9865-165a35c2618d");
    let preferred_username = test_actor.preferred_username.clone();
    delete_test_actor(test_actor);

    match actor::get_local_actor_by_preferred_username(&database, &preferred_username) {
        Ok(_) => assert!(false, "Actor was not deleted!"),
        Err(_) => assert!(true),
    }
}
