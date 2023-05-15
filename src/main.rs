#[macro_use]
extern crate rocket;

mod cors;
mod handlers;
mod models;

use cors::*;
use handlers::*;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[launch]
async fn rocket() -> _ {
    pretty_env_logger::init();
    dotenvy::dotenv().ok();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL must be set."))
        .await
        .unwrap();

    let recs = sqlx::query!("SELECT * FROM questions")
        .fetch_all(&pool)
        .await
        .unwrap();

    info!("********* Question Records *********");
    info!("{:?}", recs);

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
