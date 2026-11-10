#[macro_use]
extern crate rocket;

mod subscribe;
mod users;

use ignite_panel::Ignite;
use propel_rocket::PropelView;
use propel_runtime::{Assets, RenderError};
use serde::Serialize;

/// The views compiled by `build.rs`. A release build renders them from here.
mod compiled {
    include!(concat!(env!("OUT_DIR"), "/propel_views.rs"));
}

#[derive(Serialize)]
pub(crate) struct Home {
    title: &'static str,
}

/// Renders a page. A debug build uses the dev renderer, which reads the views on
/// each request, so an edit shows on refresh. A release build renders the views
/// compiled by `build.rs`, with the registered assets. The panel registers its own
/// assets, so the app adds no line for its styling.
#[cfg(debug_assertions)]
pub(crate) fn page<C, F>(name: &str, context: &C, _compiled: F) -> PropelView
where
    C: Serialize,
    F: FnOnce(&C, &Assets) -> Result<String, RenderError>,
{
    PropelView::render(name, context)
}

/// See the debug version. A release build renders the compiled view.
#[cfg(not(debug_assertions))]
pub(crate) fn page<C, F>(_name: &str, context: &C, compiled: F) -> PropelView
where
    C: Serialize,
    F: FnOnce(&C, &Assets) -> Result<String, RenderError>,
{
    PropelView::compiled(|assets| compiled(context, assets))
}

#[get("/")]
fn index() -> PropelView {
    page(
        "home",
        &Home {
            title: "Ignite example",
        },
        compiled::home::render,
    )
}

pub(crate) fn rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .attach(
            Ignite::new()
                .resource::<users::Users>()
                // An app's own profile field: every user can set it on their profile.
                .profile_field(ignite_core::Field {
                    name: "job_title",
                    label: "Job title",
                    kind: ignite_core::FieldKind::Text,
                    required: false,
                })
                .fairing(),
        )
        .mount("/", routes![index, subscribe::form, subscribe::submit])
}

#[launch]
fn launch() -> _ {
    rocket()
}

#[cfg(test)]
mod tests {
    use super::rocket;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;

    /// Each test needs a database, from DATABASE_URL, and a secret for release builds.
    fn client() -> Client {
        if std::env::var_os("IGNITE_SECRET_KEY").is_none() {
            std::env::set_var(
                "IGNITE_SECRET_KEY",
                "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=",
            );
        }
        Client::tracked(rocket()).expect("valid rocket")
    }

    fn token(client: &Client) -> String {
        let body = client.get("/subscribe").dispatch().into_string().unwrap();
        let marker = r#"name="_token" value=""#;
        let start = body.find(marker).expect("the form has a token") + marker.len();
        let end = body[start..].find('"').unwrap() + start;
        body[start..end].to_string()
    }

    #[test]
    fn the_home_page_extends_the_panel_layout() {
        let client = client();
        let response = client.get("/").dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("Ignite example"));
        assert!(body.contains("Ignite"), "the panel's header is there");
    }

    #[test]
    fn a_subscribe_post_without_a_token_is_refused_with_419() {
        let client = client();
        token(&client);
        let response = client
            .post("/subscribe")
            .header(ContentType::Form)
            .body("name=Ada&email=ada%40example.test")
            .dispatch();

        assert_eq!(response.status().code, 419);
    }

    #[test]
    fn a_subscribe_post_with_a_wrong_token_is_refused_with_419() {
        let client = client();
        token(&client);
        let response = client
            .post("/subscribe")
            .header(ContentType::Form)
            .body("name=Ada&email=ada%40example.test&_token=wrong")
            .dispatch();

        assert_eq!(response.status().code, 419);
    }

    #[test]
    fn a_subscribe_post_with_the_token_is_accepted() {
        let client = client();
        let token = token(&client);
        let response = client
            .post("/subscribe")
            .header(ContentType::Form)
            .body(format!("name=Ada&email=ada%40example.test&_token={token}"))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.into_string().unwrap().contains("Thanks, Ada"));
    }

    #[test]
    fn an_invalid_subscription_is_shown_again_with_the_error() {
        let client = client();
        let token = token(&client);
        let response = client
            .post("/subscribe")
            .header(ContentType::Form)
            .body(format!("name=&email=not-an-email&_token={token}"))
            .dispatch();

        assert_eq!(response.status().code, 422);
        assert!(response
            .into_string()
            .unwrap()
            .contains("Please enter your name."));
    }
}
