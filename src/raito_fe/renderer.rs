use activity;
use actor;
use chrono::prelude::*;
use database;
use env;
use mastodon_api::{RegistrationForm, Status, StatusForm};
use raito_fe::{self, Authentication, LocalConfiguration, LoginForm};
use rocket::http::{Cookie, Cookies};
use rocket::request::LenientForm;
use rocket::response::Redirect;
use rocket::Rocket;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::fs;

pub fn about(configuration: LocalConfiguration, authentication: Authentication) -> Template {
    let mut context = HashMap::<String, String>::new();
    context.extend(configuration);
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());
    context.insert(
        "content".to_string(),
        fs::read_to_string("static/raito_fe/html/about.html").unwrap_or_else(|_| {
            String::from(
                "<h2>About this node</h2>
This is a placeholder text, it can be edited in \"static/raito_fe/html/about.html\"
",
            )
        }),
    );
    return Template::render("raito_fe/about", context);
}

pub fn account_by_local_id(
    configuration: LocalConfiguration,
    authentication: Authentication,
    id: String,
) -> Template {
    let mut context = HashMap::<String, String>::new();
    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    match raito_fe::api_controller::get_account(&id) {
        Ok(account) => {
            match &authentication.account {
                Some(account) => {
                    if &account.id != &id {
                        match &authentication.token {
                            Some(token) => {
                                match raito_fe::api_controller::relationships_by_token(
                                    &token,
                                    vec![id.parse::<i64>().unwrap()],
                                ) {
                                    Some(relationship) => {
                                        context.insert(
                                            String::from("account_relationship_following"),
                                            relationship[0].following.to_string(),
                                        );
                                    }
                                    None => {
                                        context.insert(
                                            String::from("account_relationship_following"),
                                            String::from(""),
                                        );
                                    }
                                }
                            }
                            None => {
                                context.insert(
                                    String::from("account_relationship_following"),
                                    String::from(""),
                                );
                            }
                        }
                    } else {
                        context.insert(
                            String::from("account_relationship_following"),
                            String::from(""),
                        );
                    }
                }
                None => {
                    context.insert(
                        String::from("account_relationship_following"),
                        String::from(""),
                    );
                }
            }

            context.insert(String::from("account_acct"), account.acct);
            context.insert(String::from("account_display_name"), account.display_name);
            context.insert(String::from("account_avatar"), account.avatar);
            context.insert(
                String::from("account_followers_count"),
                account.followers_count.to_string(),
            );
            context.insert(
                String::from("account_following_count"),
                account.following_count.to_string(),
            );
            context.insert(String::from("account_header"), account.header);
            context.insert(String::from("account_id"), account.id.clone());
            context.insert(String::from("account_note"), account.note);
            context.insert(
                String::from("account_statuses_count"),
                account.statuses_count.to_string(),
            );
            context.insert(
                String::from("account_timeline"),
                user_timeline(configuration, authentication, account.id),
            );

            return Template::render("raito_fe/account", context);
        }
        Err(_) => Template::render("raito_fe/index", context),
    }
}

pub fn account_by_username(
    configuration: LocalConfiguration,
    authentication: Authentication,
    username: String,
) -> Template {
    let database = database::establish_connection();

    let mut context = HashMap::<String, String>::new();
    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());
    match actor::get_local_actor_by_preferred_username(&database, &username) {
        Ok(actor) => account_by_local_id(configuration, authentication, actor.id.to_string()),
        Err(_) => Template::render("raito_fe/index", context),
    }
}

pub fn account_follow(
    configuration: LocalConfiguration,
    authentication: Authentication,
    id: i64,
    unfollow: bool,
) -> Template {
    let _database = database::establish_connection();

    let mut context = HashMap::<String, String>::new();
    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    match &authentication.token {
        Some(token) => {
            if unfollow {
                raito_fe::api_controller::unfollow(token, id);
            } else {
                raito_fe::api_controller::follow(token, id);
            }

            return account_by_local_id(configuration, authentication, id.to_string());
        }
        None => return account_by_local_id(configuration, authentication, id.to_string()),
    }
}

pub fn compose(
    configuration: LocalConfiguration,
    authentication: Authentication,
    in_reply_to: Option<i64>,
) -> Template {
    let mut context = HashMap::<String, String>::new();
    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    if authentication.account.is_none() {
        return Template::render("raito_fe/infoscreen", context);
    } else {
        match in_reply_to {
            Some(head_status_id) => {
                let renderer = rocket::ignite().attach(Template::fairing());
                match raito_fe::api_controller::get_status(head_status_id.to_string()) {
                    Ok(status) => context.insert(
                        String::from("head_status"),
                        format!(
                            "<input type=\"hidden\" name=\"in_reply_to_id\" value=\"{}\">{}",
                            status.id.clone(),
                            raw_status(configuration, &authentication, status, &renderer)
                        ),
                    ),
                    Err(_) => context.insert(String::from("head_status"), String::from("")),
                };
            }
            None => {
                context.insert(String::from("head_status"), String::from(""));
            }
        }

        return Template::render("raito_fe/status_post", context);
    }
}

pub fn compose_post(
    configuration: LocalConfiguration,
    authentication: Authentication,
    form: LenientForm<StatusForm>,
) -> Redirect {
    let mut context = HashMap::<String, String>::new();
    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    match &authentication.token {
        Some(token) => {
            raito_fe::api_controller::post_status(form, &token);
            return Redirect::to("/timeline/home");
        }
        None => return Redirect::to("/"),
    }
}

pub fn conversation(
    configuration: LocalConfiguration,
    authentication: Authentication,
    id: String,
) -> Template {
    let mut context = HashMap::<String, String>::new();
    let rocket_renderer = rocket::ignite().attach(Template::fairing());

    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    match raito_fe::api_controller::get_status(id.clone()) {
        Ok(status) => {
            let mut renderered_statuses: Vec<String> = vec![];
            let mut parent_statuses: Vec<Status> = vec![];
            let mut child_statuses: Vec<Status> = vec![];
            let mut timeline_parameters = HashMap::<String, String>::new();

            match raito_fe::api_controller::get_status_context(id) {
                Ok(context) => {
                    parent_statuses =
                        serde_json::from_value(context["ancestors"].to_owned()).unwrap();
                    child_statuses =
                        serde_json::from_value(context["descendants"].to_owned()).unwrap();
                }
                Err(()) => (),
            }

            for parent in parent_statuses {
                renderered_statuses.push(raw_status(
                    configuration.clone(),
                    &authentication,
                    parent,
                    &rocket_renderer,
                ));
            }
            renderered_statuses.push(raw_status(
                configuration.clone(),
                &authentication,
                status,
                &rocket_renderer,
            ));
            for child in child_statuses {
                renderered_statuses.push(raw_status(
                    configuration.clone(),
                    &authentication,
                    child,
                    &rocket_renderer,
                ));
            }
            context.insert(String::from("timeline_name"), String::from("Conversation"));
            timeline_parameters.insert(String::from("statuses"), renderered_statuses.join(""));
            timeline_parameters.extend(configuration.clone());
            timeline_parameters.extend(prepare_authentication_context(&authentication));
            context.insert(
                String::from("timeline"),
                Template::show(
                    &rocket_renderer,
                    "raito_fe/components/timeline",
                    timeline_parameters,
                )
                .unwrap(),
            );

            return Template::render("raito_fe/timeline_view", context);
        }
        Err(_) => Template::render("raito_fe/index", context),
    }
}
// Note: This case only occurs if the Raito-FE is set as the main UI of Kibou
pub fn conversation_by_uri(
    configuration: LocalConfiguration,
    authentication: Authentication,
    id: String,
) -> Template {
    let database = database::establish_connection();
    let mut context = HashMap::<String, String>::new();
    let object_id = format!(
        "{}://{}/objects/{}",
        env::get_value(String::from("endpoint.base_scheme")),
        env::get_value(String::from("endpoint.base_domain")),
        id
    );

    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    match activity::get_ap_object_by_id(&database, &object_id) {
        Ok(activity) => conversation(configuration, authentication, activity.id.to_string()),
        Err(_) => Template::render("raito_fe/index", context),
    }
}

pub fn home_timeline(
    configuration: LocalConfiguration,
    authentication: Authentication,
) -> Template {
    let mut context = HashMap::<String, String>::new();
    let rocket_renderer = rocket::ignite().attach(Template::fairing());
    let mut timeline_parameters = HashMap::<String, String>::new();

    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    match &authentication.token {
        Some(token) => {
            if configuration.0.get("javascript_enabled").unwrap() == "true" {
                context.insert(
                    String::from("timeline"),
                    Template::show(
                        &rocket_renderer,
                        "raito_fe/components/timeline",
                        timeline_parameters,
                    )
                    .unwrap(),
                );
                return Template::render("raito_fe/timeline_view", context);
            } else {
                match raito_fe::api_controller::home_timeline(&format!("Bearer: {}", token)) {
                    Ok(statuses) => {
                        let mut renderered_statuses: Vec<String> = vec![];
                        for status in statuses {
                            renderered_statuses.push(raw_status(
                                configuration.clone(),
                                &authentication,
                                status,
                                &rocket_renderer,
                            ));
                        }

                        context
                            .insert(String::from("timeline_name"), String::from("Home Timeline"));
                        timeline_parameters.extend(configuration.clone());

                        timeline_parameters
                            .insert(String::from("statuses"), renderered_statuses.join(""));
                        context.insert(
                            String::from("timeline"),
                            Template::show(
                                &rocket_renderer,
                                "raito_fe/components/timeline",
                                timeline_parameters,
                            )
                            .unwrap(),
                        );

                        return Template::render("raito_fe/timeline_view", context);
                    }
                    Err(_) => Template::render("raito_fe/index", context),
                }
            }
        }
        None => return public_timeline(configuration, authentication, false),
    }
}

pub fn index(configuration: LocalConfiguration, authentication: Authentication) -> Template {
    let mut context = HashMap::<String, String>::new();

    match &authentication.account {
        Some(_account) => return home_timeline(configuration, authentication),
        None => {
            context.extend(configuration);
            context.extend(prepare_authentication_context(&authentication));
            context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());
            return Template::render("raito_fe/infoscreen", context);
        }
    }
}

pub fn login(configuration: LocalConfiguration, authentication: Authentication) -> Template {
    let mut context = HashMap::<String, String>::new();
    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    if authentication.account.is_none() {
        return Template::render("raito_fe/login", context);
    } else {
        return public_timeline(configuration, authentication, false);
    }
}

pub fn login_post(
    configuration: LocalConfiguration,
    authentication: Authentication,
    mut cookies: Cookies,
    form: LenientForm<LoginForm>,
) -> Result<Redirect, Template> {
    let mut context = HashMap::<String, String>::new();
    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    if authentication.account.is_none() {
        match raito_fe::api_controller::login(form) {
            Some(token) => {
                cookies.add_private(Cookie::new("oauth_token", token));
                return Ok(Redirect::to("/timeline/home"));
            }
            None => Err(Template::render("raito_fe/login", context)),
        }
    } else {
        return Ok(Redirect::to("/timeline/home"));
    }
}

pub fn public_timeline(
    configuration: LocalConfiguration,
    authentication: Authentication,
    local: bool,
) -> Template {
    let rocket_renderer = rocket::ignite().attach(Template::fairing());
    let mut context = HashMap::<String, String>::new();
    let mut timeline_parameters = HashMap::<String, String>::new();

    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());
    if local {
        context.insert(
            String::from("timeline_name"),
            String::from("Public Timeline"),
        );
    } else {
        context.insert(
            String::from("timeline_name"),
            String::from("Global Timeline"),
        );
    }

    timeline_parameters.extend(configuration.clone());

    if configuration.0.get("javascript_enabled").unwrap() == "true" {
        context.insert(
            String::from("timeline"),
            Template::show(
                &rocket_renderer,
                "raito_fe/components/timeline",
                timeline_parameters,
            )
            .unwrap(),
        );
        return Template::render("raito_fe/timeline_view", context);
    } else {
        match raito_fe::api_controller::get_public_timeline(local) {
            Ok(statuses) => {
                let mut renderered_statuses: Vec<String> = vec![];
                for status in statuses {
                    renderered_statuses.push(raw_status(
                        configuration.clone(),
                        &authentication,
                        status,
                        &rocket_renderer,
                    ));
                }

                timeline_parameters.insert(String::from("statuses"), renderered_statuses.join(""));
                context.insert(
                    String::from("timeline"),
                    Template::show(
                        &rocket_renderer,
                        "raito_fe/components/timeline",
                        timeline_parameters,
                    )
                    .unwrap(),
                );
                return Template::render("raito_fe/timeline_view", context);
            }
            Err(_) => Template::render("raito_fe/index", context),
        }
    }
}

pub fn raw_status(
    configuration: LocalConfiguration,
    authentication: &Authentication,
    status: Status,
    rocket: &Rocket,
) -> String {
    let mut context = prepare_status_context(status);
    context.extend(configuration);
    context.extend(prepare_authentication_context(authentication));
    return Template::show(rocket, "raito_fe/components/status", context).unwrap();
}

pub fn register_post(
    configuration: LocalConfiguration,
    authentication: Authentication,
    mut cookies: Cookies,
    form: LenientForm<RegistrationForm>,
) -> Result<Redirect, Template> {
    let mut context = HashMap::<String, String>::new();
    context.extend(configuration.clone());
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    if authentication.account.is_none() {
        match raito_fe::api_controller::register(form) {
            Some(token) => {
                cookies.add_private(Cookie::new("oauth_token", token));
                return Ok(Redirect::to("/timeline/home"));
            }
            None => return Err(Template::render("raito_fe/infoscreen", context)),
        }
    } else {
        return Ok(Redirect::to("/timeline/home"));
    }
}

pub fn settings(configuration: LocalConfiguration, authentication: Authentication) -> Template {
    let mut context = HashMap::<String, String>::new();
    context.extend(configuration);
    context.extend(prepare_authentication_context(&authentication));
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());
    return Template::render("raito_fe/settings", context);
}

pub fn user_timeline(
    configuration: LocalConfiguration,
    authentication: Authentication,
    id: String,
) -> String {
    let mut context = HashMap::<String, String>::new();

    match raito_fe::api_controller::get_user_timeline(id) {
        Ok(statuses) => {
            let mut renderered_statuses: Vec<String> = vec![];
            let rocket_renderer = rocket::ignite().attach(Template::fairing());
            for status in statuses {
                renderered_statuses.push(raw_status(
                    configuration.clone(),
                    &authentication,
                    status,
                    &rocket_renderer,
                ));
            }

            context.extend(configuration);
            context.extend(prepare_authentication_context(&authentication));
            context.insert(String::from("statuses"), renderered_statuses.join(""));
            context.insert(String::from("timeline_name"), String::from("User Timeline"));

            return Template::show(&rocket_renderer, "raito_fe/components/timeline", context)
                .unwrap();
        }
        Err(_) => String::from(""),
    }
}

fn prepare_authentication_context(authentication: &Authentication) -> HashMap<String, String> {
    let mut context = HashMap::<String, String>::new();

    match &authentication.account {
        Some(account) => {
            context.insert(String::from("authenticated_account"), true.to_string());
            context.insert(
                String::from("authenticated_account_display_name"),
                (*account.display_name).to_string(),
            );
            context.insert(
                String::from("authenticated_account_avatar"),
                (*account.avatar).to_string(),
            );
            context.insert(
                String::from("authenticated_account_id"),
                (*account.id).to_string(),
            );
        }
        None => {
            context.insert(String::from("authenticated_account"), false.to_string());
            context.insert(
                String::from("authenticated_account_display_name"),
                String::from("Guest"),
            );
            context.insert(
                String::from("authenticated_account_avatar"),
                String::from("/static/assets/default_avatar.png"),
            );
            context.insert(String::from("authenticated_account_id"), String::from(""));
        }
    }

    return context;
}

fn prepare_status_context(status: Status) -> HashMap<String, String> {
    let mut context = HashMap::<String, String>::new();

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

    context.insert(String::from("status_account_acct"), status.account.acct);
    context.insert(String::from("status_account_avatar"), status.account.avatar);
    context.insert(
        String::from("status_account_displayname"),
        status.account.display_name,
    );
    context.insert(
        String::from("status_account_url"),
        format!("/account/{}", status.account.id),
    );
    context.insert(String::from("status_content"), status.content);
    context.insert(String::from("status_created_at"), date);
    context.insert(String::from("status_favourites_count"), favourites_count);
    context.insert(String::from("status_id"), status.id.to_string());
    context.insert(
        String::from("status_reblog"),
        status.reblog.is_some().to_string(),
    );

    let mut media_context: Vec<String> = Vec::new();
    for attachment in status.media_attachments {
        media_context.push(format!("<img src=\"{}\">", attachment.url));
    }

    context.insert(
        String::from("status_media_attachments"),
        media_context.join(""),
    );

    match status.reblog {
        Some(reblog_status) => {
            let reblog: Status = serde_json::from_value(reblog_status).unwrap();
            context.insert(String::from("reblog_account_acct"), reblog.account.acct);
            context.insert(String::from("reblog_account_avatar"), reblog.account.avatar);
            context.insert(
                String::from("reblog_account_url"),
                format!("/account/{}", reblog.id),
            );
            context.insert(String::from("reblog_content"), reblog.content);
        }
        None => {
            context.insert(String::from("reblog_account_acct"), String::from(""));
            context.insert(String::from("reblog_account_avatar"), String::from(""));
            context.insert(String::from("reblog_account_url"), String::from(""));
            context.insert(String::from("reblog_content"), String::from(""));
        }
    }
    context.insert(String::from("status_reblogs_count"), shares_count);
    context.insert(String::from("status_replies_count"), replies_count);
    context.insert(String::from("status_uri"), status.uri);
    context.insert(String::from("status_url"), format!("/status/{}", status.id));

    return context;
}
