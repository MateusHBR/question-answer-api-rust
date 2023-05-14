use chrono::prelude::{DateTime, Local};
use rocket::serde::json::Json;
use std::{time::SystemTime, vec};

use crate::models::*;

#[post("/answer", data = "<answer>")]
pub async fn create_answer(answer: Json<Answer>) -> Json<AnswerDetail> {
    let now = SystemTime::now();
    let now: DateTime<Local> = now.into();

    Json(AnswerDetail {
        answer_uuid: "answer.uuid".to_owned(),
        question_uuid: answer.0.question_uuid,
        content: answer.0.content,
        created_at: now.to_rfc3339(),
    })
}

#[get("/answers/<question_uuid>")]
pub async fn read_answers(question_uuid: String) -> Json<Vec<AnswerDetail>> {
    let now = SystemTime::now();
    let now: DateTime<Local> = now.into();

    Json(vec![
        AnswerDetail {
            answer_uuid: "answer1".to_owned(),
            question_uuid: "question1".to_owned(),
            content: "First answer".to_owned(),
            created_at: now.to_rfc3339(),
        },
        AnswerDetail {
            answer_uuid: "answer2".to_owned(),
            question_uuid: "question2".to_owned(),
            content: "Second answer".to_owned(),
            created_at: now.to_rfc3339(),
        },
    ])
}

#[delete("/answer/<answer_uuid>")]
pub async fn delete_answer(answer_uuid: String) {
    println!("Delete answer with id: {}", answer_uuid)
    // Todo: implement
}
