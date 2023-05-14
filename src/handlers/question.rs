use chrono::{DateTime, Local};
use rocket::serde::json::Json;
use std::time::SystemTime;

use crate::models::*;

#[post("/question", data = "<question>")]
pub async fn create_question(question: Json<Question>) -> Json<QuestionDetail> {
    let now = SystemTime::now();
    let now: DateTime<Local> = now.into();

    Json(QuestionDetail {
        question_uuid: "Question id".to_owned(),
        title: question.0.title,
        description: question.0.description,
        created_at: now.to_rfc3339(),
    })
}

#[get("/questions")]
pub async fn read_questions() -> Json<Vec<QuestionDetail>> {
    let now = SystemTime::now();
    let now: DateTime<Local> = now.into();

    Json(vec![
        QuestionDetail {
            question_uuid: "q1".to_owned(),
            title: "First question".to_owned(),
            description: "First question".to_owned(),
            created_at: now.to_rfc3339(),
        },
        QuestionDetail {
            question_uuid: "q2".to_owned(),
            title: "Second question".to_owned(),
            description: "Second question".to_owned(),
            created_at: now.to_rfc3339(),
        },
    ])
}

#[delete("/question/<question_uuid>")]
pub async fn delete_question(question_uuid: String) {
    println!("Delete question with id: {}", question_uuid);
    // Todo: implement
}
