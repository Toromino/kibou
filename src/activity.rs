use database::models::{InsertActivity, QueryActivity};
use database::schema::activities;
use database::schema::activities::dsl::*;
use diesel::ExpressionMethods;
use diesel::sql_query;
use diesel::query_dsl::QueryDsl;
use diesel::query_dsl::RunQueryDsl;
use diesel::pg::PgConnection;
use serde_json;

pub struct Activity
{
    pub id: i64,
    pub data: serde_json::Value,
    pub actor: String
}

pub fn get_activity_by_id(db_connection: &PgConnection, activity_id: i64) -> Result<Activity, diesel::result::Error>
{
    match activities
        .filter(id.eq(activity_id))
        .limit(1)
        .first::<QueryActivity>(db_connection)
        {
            Ok(activity) => Ok(serialize_activity(activity)),
            Err(e) => Err(e)
        }
}

/// # Note
///
/// [TODO]
/// Originally I did not want any protocol-specific functions in this file, so it might be
/// reasonable to move ActivityPub-specfic database querys into their own module
pub fn get_ap_object_by_id(db_connection: &PgConnection, object_id: &str) -> Result<Activity, diesel::result::Error>
{
    match sql_query(format!("SELECT * FROM activities WHERE data->'object'->>'id' = '{}' LIMIT 1;", object_id))
         .clone()
         .load::<QueryActivity>(db_connection)
     {
         Ok(activity) => {
             if !activity.is_empty()
             {
                 let new_activity = std::borrow::ToOwned::to_owned(&activity[0]);
                 Ok(serialize_activity(new_activity))
             } else { Err(diesel::result::Error::NotFound) }
         },
         Err(e) => Err(e),
     }
}

pub fn get_ap_object_replies_by_id(db_connection: &PgConnection, object_id: &str) -> Result<Vec<Activity>, diesel::result::Error>
{
    match sql_query(format!("SELECT * FROM activities WHERE data->'object'->>'inReplyTo' = '{}';", object_id))
         .clone()
         .load::<QueryActivity>(db_connection)
     {
         Ok(activity) => {

             let mut serialized_activites: Vec<Activity> = vec![];

             for object in activity
             {
                 serialized_activites.push(serialize_activity(object));
             }

             Ok(serialized_activites)
         },
         Err(e) => Err(e),
     }
}

pub fn count_ap_object_replies_by_id(db_connection: &PgConnection, object_id: &str) -> Result<usize, diesel::result::Error>
{
    match sql_query(format!("SELECT * FROM activities WHERE data->'object'->>'inReplyTo' = '{}';", object_id))
         .clone()
         .load::<QueryActivity>(db_connection)
     {
         Ok(activity) => Ok(activity.len()),
         Err(e) => Err(e),
     }
}

pub fn count_ap_object_reactions_by_id(db_connection: &PgConnection, object_id: &str, reaction: &str) -> Result<usize, diesel::result::Error>
{
    match sql_query(format!("SELECT * FROM activities WHERE data->>'type' = '{reaction_type}' AND data->>'object'= '{id}';",
    reaction_type = reaction,
    id = object_id))
         .clone()
         .load::<QueryActivity>(db_connection)
     {
         Ok(activity) => Ok(activity.len()),
         Err(e) => Err(e),
     }
}

fn serialize_activity(sql_activity: QueryActivity) -> Activity
{
    Activity {id: sql_activity.id, data: sql_activity.data, actor: sql_activity.actor_uri }
}

fn deserialize_activity(activity: &Activity) -> InsertActivity
{
    InsertActivity { data: &activity.data, actor_uri: &activity.actor }
}

pub fn insert_activity(db_connection: &PgConnection, activity: Activity)
{
    let new_activity = deserialize_activity(&activity);

    diesel::insert_into(activities::table)
    .values(new_activity)
    .execute(db_connection)
    .expect("Error creating activity");
}

pub fn delete_ap_object_by_id (db_connection: &PgConnection, object_id: String)
{
    sql_query(format!("DELETE FROM activities WHERE data->'object'->>'id' = '{}';", object_id))
    .execute(db_connection);
}
