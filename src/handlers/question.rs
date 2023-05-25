use super::{
    private::{self},
    APIError,
};
use crate::models::*;
use crate::persistence::question_dao::QuestionDao;
use rocket::{serde::json::Json, State};

#[post("/question", data = "<question>")]
pub async fn create_question(
    question: Json<Question>,
    question_dao: &State<Box<dyn QuestionDao + Sync + Send>>,
) -> Result<Json<QuestionDetail>, APIError> {
    // let now = SystemTime::now();
    // let now: DateTime<Local> = now.into();
    let result = private::create_question(question.0, question_dao)
        .await
        .map_err(|err| APIError::from(err))?;

    Ok(Json(result))
}

#[get("/questions")]
pub async fn get_questions(
    question_dao: &State<Box<dyn QuestionDao + Sync + Send>>,
) -> Result<Json<Vec<QuestionDetail>>, APIError> {
    let result = private::get_questions(question_dao)
        .await
        .map_err(|err| APIError::from(err))?;

    Ok(Json(result))
}

#[delete("/question/<question_uuid>")]
pub async fn delete_question(
    question_uuid: String,
    question_dao: &State<Box<dyn QuestionDao + Sync + Send>>,
) -> Result<(), APIError> {
    private::delete_question(question_uuid, question_dao)
        .await
        .map_err(|err| APIError::from(err))?;

    Ok(())
}
