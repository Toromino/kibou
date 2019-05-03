use activity;
use database;
use env;
use mastodon_api::Status;
use raito_fe;
use raito_fe::status::render_raw_status;
use rocket_contrib::templates::Template;
use std::collections::HashMap;

pub fn get_conversation_by_uri(id: String) -> Template {
    let database = database::establish_connection();
    let mut template_parameters = HashMap::<String, String>::new();
    let object_id = format!(
        "{}://{}/objects/{}",
        env::get_value(String::from("endpoint.base_scheme")),
        env::get_value(String::from("endpoint.base_domain")),
        id
    );

    template_parameters.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    match activity::get_ap_object_by_id(&database, &object_id) {
        Ok(activity) => render_conversation(activity.id.to_string()),
        Err(_) => Template::render("raito_fe/index", template_parameters),
    }
}

pub fn render_conversation(id: String) -> Template {
    let mut template_parameters = HashMap::<String, String>::new();
    let rocket_renderer = rocket::ignite().attach(Template::fairing());

    template_parameters.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

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
                renderered_statuses.push(render_raw_status(parent, &rocket_renderer));
            }
            renderered_statuses.push(render_raw_status(status, &rocket_renderer));
            for child in child_statuses {
                renderered_statuses.push(render_raw_status(child, &rocket_renderer));
            }
            template_parameters.insert(String::from("timeline_name"), String::from("Conversation"));
            timeline_parameters.insert(String::from("statuses"), renderered_statuses.join(""));
            template_parameters.insert(
                String::from("timeline"),
                Template::show(
                    &rocket_renderer,
                    "raito_fe/components/timeline",
                    timeline_parameters,
                )
                .unwrap(),
            );

            return Template::render("raito_fe/timeline_view", template_parameters);
        }
        Err(_) => Template::render("raito_fe/index", template_parameters),
    }
}

pub fn render_home_timeline() -> Template {
    let mut template_parameters = HashMap::<String, String>::new();
    template_parameters.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    match raito_fe::api_controller::get_public_timeline(false) {
        Ok(statuses) => {
            let mut renderered_statuses: Vec<String> = vec![];
            let rocket_renderer = rocket::ignite().attach(Template::fairing());
            let mut timeline_parameters = HashMap::<String, String>::new();
            for status in statuses {
                renderered_statuses.push(render_raw_status(status, &rocket_renderer));
            }

            template_parameters
                .insert(String::from("timeline_name"), String::from("Home Timeline"));

            timeline_parameters.insert(String::from("statuses"), renderered_statuses.join(""));
            template_parameters.insert(
                String::from("timeline"),
                Template::show(
                    &rocket_renderer,
                    "raito_fe/components/timeline",
                    timeline_parameters,
                )
                .unwrap(),
            );

            return Template::render("raito_fe/timeline_view", template_parameters);
        }
        Err(_) => Template::render("raito_fe/index", template_parameters),
    }
}

pub fn render_public_timeline(local: bool) -> Template {
    let mut template_parameters = HashMap::<String, String>::new();
    template_parameters.insert("stylesheet".to_string(), raito_fe::get_stylesheet());

    match raito_fe::api_controller::get_public_timeline(local) {
        Ok(statuses) => {
            let mut renderered_statuses: Vec<String> = vec![];
            let rocket_renderer = rocket::ignite().attach(Template::fairing());
            let mut timeline_parameters = HashMap::<String, String>::new();
            for status in statuses {
                renderered_statuses.push(render_raw_status(status, &rocket_renderer));
            }

            if local {
                template_parameters.insert(String::from("timeline_name"), String::from("Public"));
            } else {
                template_parameters.insert(String::from("timeline_name"), String::from("Global"));
            }
            timeline_parameters.insert(String::from("statuses"), renderered_statuses.join(""));
            template_parameters.insert(
                String::from("timeline"),
                Template::show(
                    &rocket_renderer,
                    "raito_fe/components/timeline",
                    timeline_parameters,
                )
                .unwrap(),
            );

            return Template::render("raito_fe/timeline_view", template_parameters);
        }
        Err(_) => Template::render("raito_fe/index", template_parameters),
    }
}

pub fn get_user_timeline(id: String) -> String {
    let mut template_parameters = HashMap::<String, String>::new();

    match raito_fe::api_controller::get_user_timeline(id) {
        Ok(statuses) => {
            let mut renderered_statuses: Vec<String> = vec![];
            let rocket_renderer = rocket::ignite().attach(Template::fairing());
            for status in statuses {
                renderered_statuses.push(render_raw_status(status, &rocket_renderer));
            }

            template_parameters.insert(String::from("statuses"), renderered_statuses.join(""));
            template_parameters
                .insert(String::from("timeline_name"), String::from("User Timeline"));

            return Template::show(
                &rocket_renderer,
                "raito_fe/components/timeline",
                template_parameters,
            )
            .unwrap();
        }
        Err(_) => String::from(""),
    }
}
