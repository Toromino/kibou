use database;
use oauth;
use oauth::application::Application as OAuthApplication;
use rocket_contrib;

#[derive(FromForm)]
pub struct ApplicationForm {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#application
    // - https://docs.joinmastodon.org/api/rest/apps/#post-api-v1-apps
    pub client_name: String,
    pub redirect_uris: String,
    pub scopes: String,
    pub website: Option<String>,
}

pub fn create_application(application: OAuthApplication) -> rocket_contrib::json::JsonValue {
    let database = database::establish_connection();
    let oauth_app: OAuthApplication = oauth::application::create(&database, application);
    rocket_contrib::json!({
        "name": oauth_app.client_name.unwrap_or_default(),
        "website": oauth_app.website,
        "client_id": oauth_app.client_id,
        "client_secret": oauth_app.client_secret,
        "redirect_uri": oauth_app.redirect_uris,
        "id": oauth_app.id
    })
}
