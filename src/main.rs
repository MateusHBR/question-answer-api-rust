#[macro_use]
extern crate rocket;

mod cors;
mod handlers;
mod models;

use cors::*;
use handlers::*;

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                answer::create_answer,
                answer::read_answers,
                answer::delete_answer,
                question::create_question,
                question::read_questions,
                question::delete_question,
            ],
        )
        .attach(CORS)
}
