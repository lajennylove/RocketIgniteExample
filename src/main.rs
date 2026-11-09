#[macro_use]
extern crate rocket;

mod users;

use diesel::mysql::MysqlConnection;
use diesel::Connection;
use ignite_panel::Ignite;
use propel_rocket::PropelView;
use serde::Serialize;

#[derive(Serialize)]
struct Home {
    title: &'static str,
}

#[get("/")]
fn index() -> PropelView {
    PropelView::render(
        "home",
        &Home {
            title: "Ignite example",
        },
    )
}

#[launch]
fn rocket() -> _ {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");
    let mut conn = MysqlConnection::establish(&url).expect("could not connect to the database");
    ignite_panel::run_migrations(&mut conn).expect("could not run the panel's migrations");

    rocket::build()
        .attach(Ignite::new().resource::<users::Users>().fairing())
        .mount("/", routes![index])
}
