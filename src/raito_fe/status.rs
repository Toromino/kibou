use chrono::prelude::*;
use mastodon_api::Status;
use raito_fe;
use rocket::Rocket;
use rocket_contrib::templates::Template;
use std::collections::HashMap;

fn prepare_status(status: Status) -> HashMap<String, String> {
    let mut template_parameters = HashMap::<String, String>::new();

    let date: String;
    let favourites_count: String;
    let replies_count: String;
    let shares_count: String;

    match DateTime::parse_from_rfc3339(&status.created_at) {
        Ok(parsed_date) => date = parsed_date.format("%B %d, %Y, %H:%M:%S").to_string(),
        Err(_) => date = String::from("Unknown date"),
    }

    if status.favourites_count > 0 {
        favourites_count = status.favourites_count.to_string()
    } else {
        favourites_count = String::from("")
    }

    if status.replies_count > 0 {
        replies_count = status.replies_count.to_string()
    } else {
        replies_count = String::from("")
    }

    if status.reblogs_count > 0 {
        shares_count = status.reblogs_count.to_string()
    } else {
        shares_count = String::from("")
    }

    template_parameters.insert(String::from("status_account_acct"), status.account.acct);
    template_parameters.insert(String::from("status_account_avatar"), status.account.avatar);
    template_parameters.insert(
        String::from("status_account_displayname"),
        status.account.display_name,
    );
    template_parameters.insert(
        String::from("status_account_url"),
        format!("/account/{}", status.account.id),
    );
    template_parameters.insert(String::from("status_content"), status.content);
    template_parameters.insert(String::from("status_created_at"), date);
    template_parameters.insert(String::from("status_favourites_count"), favourites_count);
    template_parameters.insert(String::from("status_reblogs_count"), shares_count);
    template_parameters.insert(String::from("status_replies_count"), replies_count);
    template_parameters.insert(String::from("status_uri"), status.uri);
    template_parameters.insert(String::from("status_url"), format!("/status/{}", status.id));

    return template_parameters;
}

pub fn render_status_by_local_id(id: String) -> Template {
    let mut template_parameters = HashMap::<String, String>::new();

    match raito_fe::api_controller::get_status(id) {
        Ok(status) => {
            template_parameters = prepare_status(status);
            template_parameters.insert("stylesheet".to_string(), raito_fe::get_stylesheet());
            return Template::render("raito_fe/status_view", template_parameters);
        }
        Err(_) => Template::render("raito_fe/index", template_parameters),
    }
}

pub fn render_raw_status(status: Status, rocket: &Rocket) -> String {
    let template_parameters = prepare_status(status);
    return Template::show(rocket, "raito_fe/components/status", template_parameters).unwrap();
}
