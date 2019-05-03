use actor;
use database;
use raito_fe;
use raito_fe::timeline::get_user_timeline;
use rocket_contrib::templates::Template;
use std::collections::HashMap;

pub fn get_account_by_local_id(id: String) -> Template {
    let mut template_parameters = HashMap::<String, String>::new();
    template_parameters.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    match raito_fe::api_controller::get_account(id) {
        Ok(account) => {
            template_parameters.insert(String::from("account_acct"), account.acct);
            template_parameters.insert(String::from("account_display_name"), account.display_name);
            template_parameters.insert(String::from("account_avatar"), account.avatar);
            template_parameters.insert(
                String::from("account_followers_count"),
                account.followers_count.to_string(),
            );
            template_parameters.insert(
                String::from("account_following_count"),
                account.following_count.to_string(),
            );
            template_parameters.insert(String::from("account_header"), account.header);
            template_parameters.insert(String::from("account_note"), account.note);
            template_parameters.insert(
                String::from("account_statuses_count"),
                account.statuses_count.to_string(),
            );

            template_parameters.insert(
                String::from("account_timeline"),
                get_user_timeline(account.id),
            );

            return Template::render("raito_fe/account", template_parameters);
        }
        Err(_) => Template::render("raito_fe/index", template_parameters),
    }
}

pub fn get_account_by_username(username: String) -> Template {
    let database = database::establish_connection();

    let mut template_parameters = HashMap::<String, String>::new();
    template_parameters.insert("stylesheet".to_string(), raito_fe::get_stylesheet());
    match actor::get_local_actor_by_preferred_username(&database, username) {
        Ok(actor) => get_account_by_local_id(actor.id.to_string()),
        Err(_) => Template::render("raito_fe/index", template_parameters),
    }
}
