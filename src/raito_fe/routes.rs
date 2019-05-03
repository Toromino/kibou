use raito_fe;
use raito_fe::account;
use raito_fe::timeline;
use rocket_contrib::templates::Template;
use std::collections::HashMap;

#[get("/")]
pub fn index() -> Template {
    timeline::render_public_timeline(false)
}

#[get("/about")]
pub fn about() -> Template {
    let mut context = HashMap::<String, String>::new();
    context.insert("stylesheet".to_string(), raito_fe::get_stylesheet());
    return Template::render("raito_fe/about", context);
}

#[get("/account/<id>", rank = 2)]
pub fn account(id: String) -> Template {
    account::get_account_by_local_id(id)
}

#[get("/actors/<handle>", rank = 2)]
pub fn actor(handle: String) -> Template {
    account::get_account_by_username(handle)
}

#[get("/objects/<id>", rank = 2)]
pub fn object(id: String) -> Template {
    timeline::get_conversation_by_uri(id)
}

#[get("/status/<id>", rank = 2)]
pub fn view_status(id: String) -> Template {
    timeline::render_conversation(id)
}

#[get("/timeline/global", rank = 2)]
pub fn global_timeline() -> Template {
    timeline::render_public_timeline(false)
}

#[get("/timeline/public", rank = 2)]
pub fn public_timeline() -> Template {
    timeline::render_public_timeline(true)
}
