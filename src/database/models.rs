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

#[derive(Queryable, PartialEq, QueryableByName, Clone)]
#[table_name = "activities"]
pub struct QueryActivityId {
    pub id: i64,
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

#[derive(Queryable, Debug)]
pub struct QueryOAuthApplication {
    pub id: i64,
    pub client_name: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: String,
    pub scopes: String,
    pub website: Option<String>,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
}

#[derive(Queryable, Debug)]
pub struct QueryOAuthAuthorization {
    pub id: i64,
    pub application: i64,
    pub actor: String,
    pub code: String,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub valid_until: NaiveDateTime,
}

#[derive(Queryable, Debug)]
pub struct QueryOauthToken {
    pub id: i64,
    pub application: i64,
    pub actor: String,
    pub access_token: String,
    pub refresh_token: String,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub valid_until: NaiveDateTime,
}
