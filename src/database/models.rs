use chrono::NaiveDateTime;
use database::schema::activities;
use database::schema::actors;

#[derive(Queryable, PartialEq, QueryableByName, Clone)]
#[table_name = "activities"]
pub struct QueryActivity {
    pub id: i64,
    pub data: serde_json::Value,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub actor_uri: String,
}

#[derive(Insertable)]
#[table_name = "activities"]
pub struct InsertActivity<'a> {
    pub data: &'a serde_json::Value,
    pub actor_uri: &'a String,
}

#[derive(Queryable, PartialEq, QueryableByName, Clone)]
#[table_name = "actors"]
pub struct QueryActor {
    pub id: i64,
    pub email: Option<String>,
    pub password: Option<String>,
    pub actor_uri: String,
    pub username: Option<String>,
    pub preferred_username: String,
    pub summary: Option<String>,
    pub inbox: Option<String>,
    pub icon: Option<String>,
    pub keys: serde_json::Value,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub local: bool,
    pub followers: serde_json::Value,
}
