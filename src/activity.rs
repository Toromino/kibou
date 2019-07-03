use database::models::{InsertActivity, QueryActivity};
use database::runtime_escape;
use database::schema::activities;
use database::schema::activities::dsl::*;
use diesel::pg::PgConnection;
use diesel::query_dsl::QueryDsl;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_query;
use diesel::ExpressionMethods;
use env;
use serde_json;
#[derive(Clone)]
pub struct Activity {
    pub id: i64,
    pub data: serde_json::Value,
    pub actor: String,
}

pub fn count_ap_object_replies_by_id(
    db_connection: &PgConnection,
    object_id: &str,
) -> Result<usize, diesel::result::Error> {
    match sql_query(format!(
        "SELECT * FROM activities WHERE data @> '{{\"object\": {{\"inReplyTo\": \"{}\"}}}}';",
        runtime_escape(object_id)
    ))
    .load::<QueryActivity>(db_connection)
    {
        Ok(activity_arr) => Ok(activity_arr.len()),
        Err(e) => Err(e),
    }
}

pub fn count_ap_object_reactions_by_id(
    db_connection: &PgConnection,
    object_id: &str,
    reaction: &str,
) -> Result<usize, diesel::result::Error> {
    match sql_query(format!(
        "SELECT * FROM activities WHERE data @> '{{\"type\": \"{reaction_type}\"}}' \
         AND data @> '{{\"object\": \"{id}\"}}';",
        reaction_type = runtime_escape(reaction),
        id = runtime_escape(object_id)
    ))
    .load::<QueryActivity>(db_connection)
    {
        Ok(activity_arr) => Ok(activity_arr.len()),
        Err(e) => Err(e),
    }
}

pub fn count_ap_notes_for_actor(
    db_connection: &PgConnection,
    actor: &str,
) -> Result<usize, diesel::result::Error> {
    match sql_query(format!(
        "SELECT * \
         FROM activities \
         WHERE data->>'type' = 'Create' \
         AND data->'object'->>'type' = 'Note' \
         AND data->>'actor' = '{actor}' \
         AND ((data->>'to')::jsonb ? 'https://www.w3.org/ns/activitystreams#Public' \
         OR (data->>'cc')::jsonb ? 'https://www.w3.org/ns/activitystreams#Public');",
        actor = runtime_escape(actor)
    ))
    .load::<QueryActivity>(db_connection)
    {
        Ok(activity_arr) => Ok(activity_arr.len()),
        Err(e) => Err(e),
    }
}

pub fn count_local_ap_notes(db_connection: &PgConnection) -> Result<usize, diesel::result::Error> {
    match sql_query(format!(
        "SELECT * \
         FROM activities \
         WHERE data->>'type' = 'Create' \
         AND data->'object'->>'type' = 'Note' \
         AND data->>'actor' LIKE '{base_scheme}://{base_domain}/%' \
         AND ((data->>'to')::jsonb ? 'https://www.w3.org/ns/activitystreams#Public' \
         OR (data->>'cc')::jsonb ? 'https://www.w3.org/ns/activitystreams#Public')",
        base_scheme = env::get_value(String::from("endpoint.base_scheme")),
        base_domain = env::get_value(String::from("endpoint.base_domain"))
    ))
    .clone()
    .load::<QueryActivity>(db_connection)
    {
        Ok(activity_arr) => Ok(activity_arr.len()),
        Err(e) => Err(e),
    }
}

pub fn get_activity_by_id(
    db_connection: &PgConnection,
    activity_id: i64,
) -> Result<Activity, diesel::result::Error> {
    match activities
        .filter(id.eq(activity_id))
        .limit(1)
        .first::<QueryActivity>(db_connection)
    {
        Ok(activity) => Ok(serialize_activity(activity)),
        Err(e) => Err(e),
    }
}

pub fn get_ap_activity_by_id(
    db_connection: &PgConnection,
    activity_id: &str,
) -> Result<Activity, diesel::result::Error> {
    match sql_query(format!(
        "SELECT * FROM activities WHERE data @> '{{\"id\": \"{}\"}}' LIMIT 1;",
        runtime_escape(activity_id)
    ))
    .clone()
    .load::<QueryActivity>(db_connection)
    {
        Ok(activity) => {
            if !activity.is_empty() {
                let new_activity = std::borrow::ToOwned::to_owned(&activity[0]);
                Ok(serialize_activity(new_activity))
            } else {
                Err(diesel::result::Error::NotFound)
            }
        }
        Err(e) => Err(e),
    }
}

pub fn get_ap_object_by_id(
    db_connection: &PgConnection,
    object_id: &str,
) -> Result<Activity, diesel::result::Error> {
    match sql_query(format!(
        "SELECT * FROM activities WHERE data @> '{{\"object\": {{\"id\": \"{}\"}}}}' LIMIT 1;",
        runtime_escape(object_id)
    ))
    .load::<QueryActivity>(db_connection)
    {
        Ok(activity) => {
            if !activity.is_empty() {
                let new_activity = std::borrow::ToOwned::to_owned(&activity[0]);
                Ok(serialize_activity(new_activity))
            } else {
                Err(diesel::result::Error::NotFound)
            }
        }
        Err(e) => Err(e),
    }
}

pub fn get_ap_object_replies_by_id(
    db_connection: &PgConnection,
    object_id: &str,
) -> Result<Vec<Activity>, diesel::result::Error> {
    match sql_query(format!(
        "SELECT * FROM activities WHERE data->'object'->>'inReplyTo' = '{}';",
        runtime_escape(object_id)
    ))
    .load::<QueryActivity>(db_connection)
    {
        Ok(activity) => {
            let mut serialized_activites: Vec<Activity> = vec![];

            for object in activity {
                serialized_activites.push(serialize_activity(object));
            }

            Ok(serialized_activites)
        }
        Err(e) => Err(e),
    }
}

pub fn type_exists_for_object_id(
    db_connection: &PgConnection,
    _type: &str,
    actor: &str,
    object_id: &str,
) -> Result<bool, diesel::result::Error> {
    match sql_query(format!(
        "SELECT * FROM activities WHERE data @> '{{\"type\": \"{}\", \"actor\": \"{}\", \"object\": \"{}\"}}' LIMIT 1;",
        _type, runtime_escape(actor), runtime_escape(object_id)
    ))
        .load::<QueryActivity>(db_connection)
        {
            Ok(activity) => {
                if !activity.is_empty() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => Err(e),
        }
}

pub fn serialize_activity(sql_activity: QueryActivity) -> Activity {
    Activity {
        id: sql_activity.id,
        data: sql_activity.data,
        actor: sql_activity.actor_uri,
    }
}

pub fn deserialize_activity(activity: &Activity) -> InsertActivity {
    InsertActivity {
        data: &activity.data,
        actor_uri: &activity.actor,
    }
}

pub fn insert_activity(db_connection: &PgConnection, activity: Activity) -> Activity {
    let new_activity = deserialize_activity(&activity);

    serialize_activity(
        diesel::insert_into(activities::table)
            .values(&new_activity)
            .get_result(db_connection)
            .expect("Error creating activity"),
    )
}

pub fn delete_ap_activity_by_id(db_connection: &PgConnection, activity_id: String) {
    sql_query(format!(
        "DELETE FROM activities WHERE data->>'id' = '{}';",
        runtime_escape(&activity_id)
    ))
    .execute(db_connection);
}

pub fn delete_ap_object_by_id(db_connection: &PgConnection, object_id: String) {
    sql_query(format!(
        "DELETE FROM activities WHERE data->'object'->>'id' = '{}';",
        runtime_escape(&object_id)
    ))
    .execute(db_connection);
}
