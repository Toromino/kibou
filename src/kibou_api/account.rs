use activitypub::controller::activity_follow;
use actor::Actor;
use actor::get_actor_by_uri;
use database;
use web_handler::federator;

pub fn follow(actor: String, object: String)
{
    let database = database::establish_connection();
    let serialized_actor: Actor = get_actor_by_uri(&database, &actor).unwrap();

    match get_actor_by_uri(&database, &object)
    {
        Ok(followee) =>
        {
            let activitypub_activity_follow = activity_follow(&actor, object);
            
            if !followee.local
            {
                federator::enqueue(serialized_actor, serde_json::json!(&activitypub_activity_follow), vec![followee.inbox.unwrap()]);
            }
        },
        Err(_) => ()
    }
}
