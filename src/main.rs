#[macro_use]
extern crate rocket;

use propel_rocket::PropelView;
use serde::Serialize;

#[derive(Serialize)]
struct Home {
    title: &'static str,
}

#[get("/")]
fn index() -> PropelView {
    PropelView::render("home", &Home { title: "Ignite example" })
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
