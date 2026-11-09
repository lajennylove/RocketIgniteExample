//! A newsletter sign-up: the app's own form. Its POST route takes `CsrfForm`, so a
//! request without the page's token is refused with 419.

use ignite_panel::{CsrfForm, CsrfToken};
use propel_rocket::PropelView;
use rocket::get;
use rocket::http::Status;
use rocket::post;
use serde::Serialize;

use crate::page;

use crate::compiled;

/// The context of the sign-up page.
#[derive(Serialize)]
pub(crate) struct Subscribe {
    pub(crate) title: &'static str,
    pub(crate) csrf_token: String,
    pub(crate) thanked: bool,
    pub(crate) has_error: bool,
    pub(crate) error: String,
    pub(crate) name: String,
    pub(crate) email: String,
}

impl Subscribe {
    fn blank(token: &CsrfToken) -> Self {
        Subscribe {
            title: "Newsletter",
            csrf_token: token.value().to_string(),
            thanked: false,
            has_error: false,
            error: String::new(),
            name: String::new(),
            email: String::new(),
        }
    }
}

#[get("/subscribe")]
pub(crate) fn form(token: CsrfToken) -> PropelView {
    let context = Subscribe::blank(&token);
    page("subscribe", &context, compiled::subscribe::render)
}

#[post("/subscribe", data = "<form>")]
pub(crate) fn submit(form: CsrfForm, token: CsrfToken) -> (Status, PropelView) {
    let name = form.get("name").unwrap_or("").trim().to_string();
    let email = form.get("email").unwrap_or("").trim().to_string();

    let problem = if name.is_empty() {
        Some("Please enter your name.")
    } else if !email.contains('@') {
        Some("Please enter an email address that looks like name@example.com.")
    } else {
        None
    };

    let mut context = Subscribe::blank(&token);
    context.name = name;
    context.email = email;
    match problem {
        Some(message) => {
            context.has_error = true;
            context.error = message.to_string();
            (
                Status::UnprocessableEntity,
                page("subscribe", &context, compiled::subscribe::render),
            )
        }
        None => {
            context.thanked = true;
            (
                Status::Ok,
                page("subscribe", &context, compiled::subscribe::render),
            )
        }
    }
}
