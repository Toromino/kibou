use actor;
use chrono::prelude::*;
use chrono::Duration;
use chrono::NaiveDateTime;
use database;
use database::models::QueryOAuthAuthorization;
use database::schema::oauth_authorizations;
use database::schema::oauth_authorizations::dsl::*;
use diesel::pg::PgConnection;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use oauth::application::get_application_by_client_id;
use openssl::bn::BigNum;
use openssl::bn::MsbOption;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use std::collections::HashMap;

pub struct Authorization {
    pub application: i64,
    pub actor: String,
    pub code: String,
    pub valid_until: String,
}

#[derive(FromForm)]
pub struct UserForm {
    pub username: String,
    pub password: String,
}

fn serialize_authorization(sql_authorization: QueryOAuthAuthorization) -> Authorization {
    Authorization {
        application: sql_authorization.application,
        actor: sql_authorization.actor,
        code: sql_authorization.code,
        valid_until: sql_authorization.valid_until.to_string(),
    }
}

pub fn get_authorization_by_code(
    db_connection: &PgConnection,
    _code: String,
) -> Result<Authorization, diesel::result::Error> {
    match oauth_authorizations
        .filter(code.eq(_code))
        .limit(1)
        .first::<QueryOAuthAuthorization>(db_connection)
    {
        Ok(authorization) => Ok(serialize_authorization(authorization)),
        Err(e) => Err(e),
    }
}

pub fn handle_user_authorization(
    user_form: UserForm,
    client_id: Option<String>,
    response_type: Option<String>,
    redirect_uri: Option<String>,
    state: Option<String>,
    styling: Option<bool>,
) -> Result<Redirect, Template> {
    let db_connection = database::establish_connection();

    if client_id.is_some() && redirect_uri.is_some() {
        match actor::authorize(&db_connection, &user_form.username, user_form.password) {
            Ok(true) => match get_application_by_client_id(&db_connection, client_id.unwrap()) {
                Ok(serialized_application) => {
                    let redirect_uri = redirect_uri.unwrap();
                    let auth_code =
                        authorize_application(user_form.username, serialized_application.id);
                    let state = match state {
                        Some(value) => format!("&state={}", value),
                        None => String::from(""),
                    };
                    let mut symbol = if redirect_uri.contains("?") {
                        "&".to_string()
                    } else {
                        "?".to_string()
                    };

                    Ok(Redirect::found(format!(
                        "{uri}{symbol}code={code}{state}",
                        uri = redirect_uri,
                        symbol = symbol,
                        code = auth_code,
                        state = state
                    )))
                }
                Err(_) => Err(user_authorization_error(
                    String::from("Invalid application credentials"),
                    styling,
                )),
            },
            _ => Err(user_authorization_error(
                String::from("Invalid login data!"),
                styling,
            )),
        }
    } else {
        Err(user_authorization_error(
            String::from("Invalid OAuth parameters"),
            styling,
        ))
    }
}

fn user_authorization_error(error_context: String, styling: Option<bool>) -> Template {
    let mut parameters = HashMap::<String, String>::new();
    parameters.insert(String::from("error_context"), error_context);
    parameters.insert(
        String::from("styling"),
        styling.unwrap_or_else(|| true).to_string(),
    );

    Template::render("oauth_authorization", parameters)
}

pub fn authorize_application(_actor: String, application_id: i64) -> String {
    let db_connection = database::establish_connection();
    let mut hex_num: BigNum = BigNum::new().unwrap();
    let utc_time: chrono::DateTime<Utc> = Utc::now();
    let expiration_date: chrono::DateTime<Utc> = utc_time + Duration::days(30);
    hex_num
        .rand(256, MsbOption::MAYBE_ZERO, true)
        .expect("Error generating authorization code");

    let new_authorization = Authorization {
        application: application_id,
        actor: _actor,
        code: hex_num.to_string(),
        valid_until: expiration_date.timestamp().to_string(),
    };

    insert(&db_connection, &new_authorization);
    return new_authorization.code;
}

fn insert(db_connection: &PgConnection, authorization: &Authorization) {
    let parsed_expiration_date: NaiveDateTime =
        chrono::NaiveDateTime::from_timestamp(authorization.valid_until.parse::<i64>().unwrap(), 0);

    let new_authorization = (
        application.eq(&authorization.application),
        actor.eq(&authorization.actor),
        code.eq(&authorization.code),
        valid_until.eq(&parsed_expiration_date),
    );

    diesel::insert_into(oauth_authorizations::table)
        .values(new_authorization)
        .execute(db_connection)
        .expect("Error creating oauth authorization");
}
