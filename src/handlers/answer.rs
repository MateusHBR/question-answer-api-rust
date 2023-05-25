use rocket::{serde::json::Json, State};

use crate::{models::*, persistence::answer_dao::AnswerDao};

use super::{
    private::{self, HandlerError},
    APIError,
};

impl From<HandlerError> for APIError {
    fn from(value: HandlerError) -> Self {
        match value {
            HandlerError::BadRequest(e) => Self::BadRequest(e),
            HandlerError::InternalError(e) => Self::InternalError(e),
        }
    }
}

#[post("/answer", data = "<answer>")]
pub async fn create_answer(
    answer: Json<Answer>,
    answer_dao: &State<Box<dyn AnswerDao + Sync + Send>>,
) -> Result<Json<AnswerDetail>, APIError> {
    let result = private::create_answer(answer.0, answer_dao)
        .await
        .map_err(|err| APIError::from(err))?;

    Ok(Json(result))
}

#[get("/answers/<question_uuid>")]
pub async fn get_answers(
    question_uuid: String,
    answer_dao: &State<Box<dyn AnswerDao + Send + Sync>>,
) -> Result<Json<Vec<AnswerDetail>>, APIError> {
    let result = private::get_answers(question_uuid, answer_dao)
        .await
        .map_err(|err| APIError::from(err))?;

    Ok(Json(result))
}

#[delete("/answer/<answer_uuid>")]
pub async fn delete_answer(
    answer_uuid: String,
    answer_dao: &State<Box<dyn AnswerDao + Send + Sync>>,
) -> Result<(), APIError> {
    private::delete_answer(answer_uuid, answer_dao)
        .await
        .map_err(|err| APIError::from(err))?;

    Ok(())
}
