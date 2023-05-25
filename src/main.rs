#[macro_use]
extern crate rocket;

mod cors;
mod handlers;
mod models;
mod persistence;

use cors::*;
use handlers::*;
use persistence::{
    answer_dao::{AnswerDao, AnswerDaoImpl},
    question_dao::{QuestionDao, QuestionDaoImpl},
};
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

    let question_dao = QuestionDaoImpl::new(pool.clone());
    let answer_dao = AnswerDaoImpl::new(pool.clone());

    rocket::build()
        .mount(
            "/",
            routes![
                question::create_question,
                question::get_questions,
                question::delete_question,
                answer::create_answer,
                answer::get_answers,
                answer::delete_answer,
            ],
        )
        .attach(CORS)
        .manage(Box::new(question_dao) as Box<dyn QuestionDao + Send + Sync>)
        .manage(Box::new(answer_dao) as Box<dyn AnswerDao + Send + Sync>)
}
