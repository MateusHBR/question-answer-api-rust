use log::error;

use crate::{
    models::{Answer, AnswerDetail, DBError, Question, QuestionDetail},
    persistence::{answer_dao::AnswerDao, question_dao::QuestionDao},
};

#[derive(Debug, PartialEq)]
pub enum HandlerError {
    BadRequest(String),
    InternalError(String),
}

impl HandlerError {
    pub fn default_internal_error() -> Self {
        HandlerError::InternalError("Something went wrong! Please try again.".to_owned())
    }
}

pub async fn create_question(
    question: Question,
    questions_dao: &Box<dyn QuestionDao + Sync + Send>,
) -> Result<QuestionDetail, HandlerError> {
    let question = questions_dao.create_question(question).await;

    match question {
        Ok(question) => Ok(question),
        Err(err) => {
            error!("Unexpected error found on create_question: {:?}", err);
            Err(HandlerError::default_internal_error())
        }
    }
}

pub async fn get_questions(
    question_dao: &Box<dyn QuestionDao + Sync + Send>,
) -> Result<Vec<QuestionDetail>, HandlerError> {
    let questions = question_dao.get_questions().await.map_err(|err| {
        error!("Failed to read questions, err: {:?}", err);
        HandlerError::default_internal_error()
    })?;

    Ok(questions)
}

pub async fn delete_question(
    question_uuid: String,
    question_dao: &Box<dyn QuestionDao + Sync + Send>,
) -> Result<(), HandlerError> {
    let result = question_dao.delete_question(question_uuid).await;

    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            error!("Error on deleting question: {}", err);

            if let DBError::InvalidUUID(s) = err {
                return Err(HandlerError::BadRequest(s));
            }

            Err(HandlerError::default_internal_error())
        }
    }
}

pub async fn create_answer(
    answer: Answer,
    answer_dao: &Box<dyn AnswerDao + Sync + Send>,
) -> Result<AnswerDetail, HandlerError> {
    let result = answer_dao.create_answer(answer).await;

    match result {
        Ok(answer) => Ok(answer),
        Err(err) => {
            error!("Something wents wrong during create_answer: {:?}", err);
            if let DBError::InvalidUUID(s) = err {
                return Err(HandlerError::BadRequest(s));
            }

            Err(HandlerError::default_internal_error())
        }
    }
}

pub async fn get_answers(
    question_uuid: String,
    answer_dao: &Box<dyn AnswerDao + Sync + Send>,
) -> Result<Vec<AnswerDetail>, HandlerError> {
    let result = answer_dao.get_answers(question_uuid).await;

    match result {
        Ok(answers) => Ok(answers),
        Err(err) => {
            error!("Error on get_answers: {:?}", err);

            if let DBError::InvalidUUID(s) = err {
                return Err(HandlerError::BadRequest(s));
            }

            Err(HandlerError::default_internal_error())
        }
    }
}

pub async fn delete_answer(
    answer_uuid: String,
    answer_dao: &Box<dyn AnswerDao + Sync + Send>,
) -> Result<(), HandlerError> {
    answer_dao.delete_answer(answer_uuid).await.map_err(|err| {
        error!("Error on delete answer: {:?}", err);

        if let DBError::InvalidUUID(s) = err {
            return HandlerError::BadRequest(s);
        }

        return HandlerError::default_internal_error();
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;

    struct QuestionDaoMock {
        create_question_response: Mutex<Option<Result<QuestionDetail, DBError>>>,
        delete_question_response: Mutex<Option<Result<(), DBError>>>,
        get_questions_response: Mutex<Option<Result<Vec<QuestionDetail>, DBError>>>,
    }

    impl QuestionDaoMock {
        fn new() -> Self {
            Self {
                create_question_response: Mutex::new(None),
                delete_question_response: Mutex::new(None),
                get_questions_response: Mutex::new(None),
            }
        }

        fn mock_create_question_response(&mut self, response: Result<QuestionDetail, DBError>) {
            self.create_question_response = Mutex::new(Some(response));
        }

        fn mock_delete_question_response(&mut self, response: Result<(), DBError>) {
            self.delete_question_response = Mutex::new(Some(response));
        }

        fn mock_get_questions_response(&mut self, response: Result<Vec<QuestionDetail>, DBError>) {
            self.get_questions_response = Mutex::new(Some(response));
        }
    }

    #[async_trait]
    impl QuestionDao for QuestionDaoMock {
        async fn create_question(&self, _: Question) -> Result<QuestionDetail, DBError> {
            self.create_question_response
                .lock()
                .await
                .take()
                .expect("create_question_response should not be None.")
        }

        async fn delete_question(&self, _: String) -> Result<(), DBError> {
            self.delete_question_response
                .lock()
                .await
                .take()
                .expect("delete_question_response should not be None.")
        }

        async fn get_questions(&self) -> Result<Vec<QuestionDetail>, DBError> {
            self.get_questions_response
                .lock()
                .await
                .take()
                .expect("get_questions_response should not be None.")
        }
    }

    struct AnswerDaoMock {
        create_answer_response: Mutex<Option<Result<AnswerDetail, DBError>>>,
        delete_answer_response: Mutex<Option<Result<(), DBError>>>,
        get_answers_response: Mutex<Option<Result<Vec<AnswerDetail>, DBError>>>,
    }

    impl AnswerDaoMock {
        fn new() -> Self {
            AnswerDaoMock {
                create_answer_response: Mutex::new(None),
                delete_answer_response: Mutex::new(None),
                get_answers_response: Mutex::new(None),
            }
        }
        fn mock_create_answer(&mut self, response: Result<AnswerDetail, DBError>) {
            self.create_answer_response = Mutex::new(Some(response));
        }
        fn mock_delete_answer(&mut self, response: Result<(), DBError>) {
            self.delete_answer_response = Mutex::new(Some(response));
        }
        fn mock_get_answers(&mut self, response: Result<Vec<AnswerDetail>, DBError>) {
            self.get_answers_response = Mutex::new(Some(response));
        }
    }

    #[async_trait]
    impl AnswerDao for AnswerDaoMock {
        async fn create_answer(&self, _: Answer) -> Result<AnswerDetail, DBError> {
            self.create_answer_response
                .lock()
                .await
                .take()
                .expect("create_answer_response should not be None.")
        }
        async fn delete_answer(&self, _: String) -> Result<(), DBError> {
            self.delete_answer_response
                .lock()
                .await
                .take()
                .expect("delete_answer_response should not be None.")
        }
        async fn get_answers(&self, _: String) -> Result<Vec<AnswerDetail>, DBError> {
            self.get_answers_response
                .lock()
                .await
                .take()
                .expect("get_answers_response should not be None.")
        }
    }

    #[tokio::test]
    async fn create_question_should_return_question() {
        let title = "title".to_owned();
        let description = "description".to_owned();
        let question = Question {
            title: title.clone(),
            description: description.clone(),
        };
        let question_detail = QuestionDetail {
            title,
            description,
            question_uuid: "uuid".to_owned(),
            created_at: "some-date".to_owned(),
        };

        let mut question_dao = QuestionDaoMock::new();
        question_dao.mock_create_question_response(Ok(question_detail.clone()));

        let question_dao: Box<dyn QuestionDao + Sync + Send> = Box::new(question_dao);

        let result = create_question(question, &question_dao).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), question_detail);
    }

    #[tokio::test]
    async fn create_question_should_return_error() {
        let question = Question {
            title: "title".to_owned(),
            description: "description".to_owned(),
        };
        let mut question_dao = QuestionDaoMock::new();
        question_dao.mock_create_question_response(Err(DBError::InvalidUUID("".to_owned())));
        let question_dao: Box<dyn QuestionDao + Sync + Send> = Box::new(question_dao);

        let result = create_question(question, &question_dao).await;
        assert!(result.is_err());
        assert_eq!(
            std::mem::discriminant(&result.unwrap_err()),
            std::mem::discriminant(&HandlerError::InternalError("".to_owned()))
        );
    }

    #[tokio::test]
    async fn get_questions_should_return_questions() {
        let questions = vec![QuestionDetail {
            title: "title".to_owned(),
            description: "description".to_owned(),
            question_uuid: "uuid".to_owned(),
            created_at: "some-date".to_owned(),
        }];
        let mut question_dao = QuestionDaoMock::new();
        question_dao.mock_get_questions_response(Ok(questions.clone()));
        let question_dao: Box<dyn QuestionDao + Sync + Send> = Box::new(question_dao);

        let result = get_questions(&question_dao).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), questions);
    }

    #[tokio::test]
    async fn get_questions_should_return_error() {
        let mut question_dao = QuestionDaoMock::new();
        question_dao.mock_get_questions_response(Err(DBError::InvalidUUID("".to_owned())));
        let question_dao: Box<dyn QuestionDao + Sync + Send> = Box::new(question_dao);
        let result = get_questions(&question_dao).await;
        assert!(result.is_err());
        assert_eq!(
            std::mem::discriminant(&result.unwrap_err()),
            std::mem::discriminant(&HandlerError::InternalError("".to_owned()))
        );
    }

    #[tokio::test]
    async fn delete_question_should_succeed() {
        let mut question_dao = QuestionDaoMock::new();
        question_dao.mock_delete_question_response(Ok(()));
        let question_dao: Box<dyn QuestionDao + Sync + Send> = Box::new(question_dao);

        let result = delete_question("question_uuid".to_owned(), &question_dao).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());
    }

    #[tokio::test]
    async fn delete_question_should_return_error() {
        let mut question_dao = QuestionDaoMock::new();
        question_dao.mock_delete_question_response(Err(DBError::InvalidUUID("".to_owned())));
        let question_dao: Box<dyn QuestionDao + Sync + Send> = Box::new(question_dao);
        let result = delete_question("question_uuid".to_owned(), &question_dao).await;
        assert!(result.is_err());
        assert_eq!(
            std::mem::discriminant(&result.unwrap_err()),
            std::mem::discriminant(&HandlerError::BadRequest("".to_owned()))
        );
    }

    #[tokio::test]
    async fn create_answer_should_return_answer() {
        let mut answer_dao = AnswerDaoMock::new();
        let answer = AnswerDetail {
            answer_uuid: "some".to_owned(),
            question_uuid: "question_uuid".to_owned(),
            content: "content".to_owned(),
            created_at: "created".to_owned(),
        };
        answer_dao.mock_create_answer(Ok(answer.clone()));
        let answer_dao: Box<dyn AnswerDao + Sync + Send> = Box::new(answer_dao);

        let result = create_answer(
            Answer {
                question_uuid: "question_id".to_owned(),
                content: "content".to_owned(),
            },
            &answer_dao,
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), answer);
    }

    #[tokio::test]
    async fn create_answer_should_return_bad_request_error() {
        let mut answer_dao = AnswerDaoMock::new();
        answer_dao.mock_create_answer(Err(DBError::InvalidUUID("".to_owned())));
        let answer_dao: Box<dyn AnswerDao + Sync + Send> = Box::new(answer_dao);
        let result = create_answer(
            Answer {
                question_uuid: "question_id".to_owned(),
                content: "content".to_owned(),
            },
            &answer_dao,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(
            std::mem::discriminant(&result.unwrap_err()),
            std::mem::discriminant(&HandlerError::BadRequest("".to_owned()))
        );
    }

    #[tokio::test]
    async fn create_answer_should_return_internal_error() {
        let mut answer_dao = AnswerDaoMock::new();
        answer_dao.mock_create_answer(Err(DBError::Other(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Oh no!",
        )))));
        let answer_dao: Box<dyn AnswerDao + Sync + Send> = Box::new(answer_dao);
        let result = create_answer(
            Answer {
                question_uuid: "question_id".to_owned(),
                content: "content".to_owned(),
            },
            &answer_dao,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(
            std::mem::discriminant(&result.unwrap_err()),
            std::mem::discriminant(&HandlerError::InternalError("".to_owned()))
        );
    }

    #[tokio::test]
    async fn get_answers_should_return_answers() {
        let answers = vec![AnswerDetail {
            answer_uuid: "some".to_owned(),
            question_uuid: "question_uuid".to_owned(),
            content: "content".to_owned(),
            created_at: "created".to_owned(),
        }];

        let mut answer_dao = AnswerDaoMock::new();
        answer_dao.mock_get_answers(Ok(answers.clone()));
        let answer_dao: Box<dyn AnswerDao + Sync + Send> = Box::new(answer_dao);

        let result = get_answers("question_uuid".to_owned(), &answer_dao).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), answers);
    }

    #[tokio::test]
    async fn get_answers_should_return_error() {
        let mut answer_dao = AnswerDaoMock::new();
        answer_dao.mock_get_answers(Err(DBError::InvalidUUID("".to_owned())));
        let answer_dao: Box<dyn AnswerDao + Sync + Send> = Box::new(answer_dao);

        let result = get_answers("question_uuid".to_owned(), &answer_dao).await;
        assert!(result.is_err());
        assert_eq!(
            std::mem::discriminant(&result.unwrap_err()),
            std::mem::discriminant(&HandlerError::BadRequest("".to_owned()))
        );
    }

    #[tokio::test]
    async fn delete_answer_should_succeed() {
        let mut answer_dao = AnswerDaoMock::new();
        answer_dao.mock_delete_answer(Ok(()));
        let answer_dao: Box<dyn AnswerDao + Sync + Send> = Box::new(answer_dao);

        let result = delete_answer("answer_uuid".to_owned(), &answer_dao).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_answer_should_return_error() {
        let mut answer_dao = AnswerDaoMock::new();
        answer_dao.mock_delete_answer(Err(DBError::InvalidUUID("".to_owned())));
        let answer_dao: Box<dyn AnswerDao + Sync + Send> = Box::new(answer_dao);

        let result = delete_answer("answer_uuid".to_owned(), &answer_dao).await;
        assert!(result.is_err());
        assert_eq!(
            std::mem::discriminant(&result.unwrap_err()),
            std::mem::discriminant(&HandlerError::BadRequest("".to_owned()))
        );
    }
}
