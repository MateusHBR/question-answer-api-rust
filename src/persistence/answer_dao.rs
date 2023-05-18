use async_trait::async_trait;
use sqlx::{types::Uuid, PgPool};

use crate::models::{postgres_error_code, Answer, AnswerDetail, DBError};

#[async_trait]
pub trait AnswerDao {
    async fn create_answer(&self, answer: Answer) -> Result<AnswerDetail, DBError>;
    async fn delete_answer(&self, answer_uuid: String) -> Result<(), DBError>;
    async fn get_answers(&self, question_uuid: String) -> Result<Vec<AnswerDetail>, DBError>;
}

pub struct AnswerDaoImpl {
    db: PgPool,
}

impl AnswerDaoImpl {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AnswerDao for AnswerDaoImpl {
    async fn create_answer(&self, answer: Answer) -> Result<AnswerDetail, DBError> {
        let question_uuid = Uuid::parse_str(&answer.question_uuid)
            .map_err(|err| DBError::InvalidUUID(err.to_string()))?;

        let result = sqlx::query!(
            "--sql
                INSERT INTO answers ( question_uuid, content )
                VALUES ( $1, $2 )
                RETURNING *
            ",
            &question_uuid,
            &answer.content,
        )
        .fetch_one(&self.db)
        .await
        .map_err(|err: sqlx::Error| match err {
            sqlx::Error::Database(err) => {
                let Some(code) = err.code() else {
                    return DBError::Other(Box::new(err))
                };

                if code.eq(postgres_error_code::FOREIGN_KEY_VIOLATION) {
                    return DBError::InvalidUUID(err.to_string());
                }

                return DBError::Other(Box::new(err));
            }
            err => DBError::Other(Box::new(err)),
        })?;

        Ok(AnswerDetail {
            answer_uuid: result.answer_uuid.to_string(),
            question_uuid: result.question_uuid.to_string(),
            content: result.content,
            created_at: result.created_at.to_string(),
        })
    }

    async fn delete_answer(&self, answer_uuid: String) -> Result<(), DBError> {
        let answer_uuid =
            Uuid::parse_str(&answer_uuid).map_err(|e| DBError::InvalidUUID(e.to_string()))?;

        sqlx::query!(
            "--sql
                DELETE from answers
                WHERE answer_uuid = $1
            ",
            answer_uuid
        )
        .execute(&self.db)
        .await
        .map_err(|err| DBError::Other(Box::new(err)))?;

        Ok(())
    }

    async fn get_answers(&self, question_uuid: String) -> Result<Vec<AnswerDetail>, DBError> {
        let question_uuid =
            Uuid::parse_str(&question_uuid).map_err(|e| DBError::InvalidUUID(e.to_string()))?;

        let result = sqlx::query!(
            "--sql
                SELECT * from answers
                WHERE question_uuid = $1
            ",
            question_uuid
        )
        .fetch_all(&self.db)
        .await
        .map_err(|err| DBError::Other(Box::new(err)))?;

        let answers = result
            .iter()
            .map(|val| AnswerDetail {
                question_uuid: val.question_uuid.to_string(),
                answer_uuid: val.answer_uuid.to_string(),
                content: val.content.clone(),
                created_at: val.created_at.to_string(),
            })
            .collect();

        Ok(answers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    use crate::{
        models::{Answer, DBError, Question},
        persistence::question_dao::{QuestionDao, QuestionDaoImpl},
    };

    #[sqlx::test]
    async fn create_answer_should_fail_with_malformed_uuid(pool: PgPool) -> Result<(), String> {
        let dao = AnswerDaoImpl::new(pool.clone());
        let result = dao
            .create_answer(Answer {
                question_uuid: "invalid-uuid".to_owned(),
                content: "content".to_owned(),
            })
            .await;

        if result.is_ok() {
            return Err(format!("Expected Err but found Ok"));
        }

        let err = result.err().unwrap();
        if let DBError::InvalidUUID(_) = err {
            return Ok(());
        }

        Err(format!(
            "Expected InvalidUUID but got: {} ",
            err.to_string()
        ))
    }

    #[sqlx::test]
    async fn create_answer_should_fail_with_non_existent_uuid(pool: PgPool) -> Result<(), String> {
        let dao = AnswerDaoImpl::new(pool.clone());

        let some_uuid = "a22abcd2-22ab-2222-a22b-2abc2a2b22cc";
        let result = dao
            .create_answer(Answer {
                question_uuid: some_uuid.to_owned(),
                content: "content".to_owned(),
            })
            .await;

        if result.is_ok() {
            return Err(format!("Expected err but found ok"));
        }
        let err = result.err().unwrap();

        match err {
            DBError::InvalidUUID(_) => Ok(()),
            err => Err(format!("Expected InvalidUUID but got: {}", err)),
        }
    }

    #[sqlx::test]
    async fn create_answer_should_fail_if_database_error_occurs(
        pool: PgPool,
    ) -> Result<(), String> {
        let dao = AnswerDaoImpl::new(pool.clone());
        pool.close().await;

        let some_uuid = "a22abcd2-22ab-2222-a22b-2abc2a2b22cc";
        let result = dao
            .create_answer(Answer {
                question_uuid: some_uuid.to_owned(),
                content: "content".to_owned(),
            })
            .await;

        if result.is_ok() {
            return Err(format!("Expected Err but got Ok"));
        };

        match result.err().unwrap() {
            DBError::Other(_) => Ok(()),
            err => Err(format!("Expected Other but got: {}", err)),
        }
    }

    #[sqlx::test]
    async fn create_answer_should_succeed(pool: PgPool) -> Result<(), String> {
        let question_dao = QuestionDaoImpl::new(pool.clone());
        let dao = AnswerDaoImpl::new(pool);

        let question = question_dao
            .create_question(Question {
                title: "title".to_owned(),
                description: "desc".to_owned(),
            })
            .await
            .unwrap();

        let result = dao
            .create_answer(Answer {
                question_uuid: question.question_uuid.clone(),
                content: "content".to_owned(),
            })
            .await;

        match result {
            Ok(value) => {
                assert_eq!(value.content, "content".to_owned());
                assert_eq!(value.question_uuid, question.question_uuid);
                Ok(())
            }
            Err(err) => Err(format!("Expected OK but found Err: {}", err)),
        }
    }

    #[sqlx::test]
    async fn delete_answer_should_fail_with_malformed_uuid(pool: PgPool) -> Result<(), String> {
        let dao = AnswerDaoImpl::new(pool);
        let result = dao
            .delete_answer("invalid_uuid".to_owned())
            .await
            .err()
            .unwrap();

        match result {
            DBError::InvalidUUID(_) => Ok(()),
            err => Err(format!("Expected InvalidUUID but got: {}", err)),
        }
    }

    #[sqlx::test]
    async fn delete_answer_should_fail_if_database_error_occurs(
        pool: PgPool,
    ) -> Result<(), String> {
        let dao = AnswerDaoImpl::new(pool.clone());
        pool.close().await;
        let some_uuid = "a22abcd2-22ab-2222-a22b-2abc2a2b22cc";
        let err = dao.delete_answer(some_uuid.to_owned()).await.err().unwrap();

        match err {
            DBError::Other(_) => Ok(()),
            err => Err(format!("Expected Other but got: {}", err)),
        }
    }

    #[sqlx::test]
    async fn delete_answer_should_works(pool: PgPool) -> Result<(), String> {
        let question_dao = QuestionDaoImpl::new(pool.clone());
        let dao = AnswerDaoImpl::new(pool.clone());

        let question = question_dao
            .create_question(Question {
                title: "title".to_owned(),
                description: "desc".to_owned(),
            })
            .await
            .unwrap();

        let answer = dao
            .create_answer(Answer {
                question_uuid: question.question_uuid,
                content: "content".to_owned(),
            })
            .await
            .unwrap();

        let result = dao.delete_answer(answer.answer_uuid).await;

        match result {
            Ok(()) => Ok(()),
            other => Err(format!("Expected Ok but got Err: {}", other.err().unwrap())),
        }
    }

    #[sqlx::test]
    async fn get_answers_should_fail_with_malformed_uuid(pool: PgPool) -> Result<(), String> {
        let dao = AnswerDaoImpl::new(pool);
        let result = dao
            .get_answers("invalid_uuid".to_owned())
            .await
            .unwrap_err();

        if let DBError::InvalidUUID(_) = result {
            return Ok(());
        }

        Err(format!("Expected InvalidUUID but got: {}", result))
    }

    #[sqlx::test]
    async fn get_answers_should_fail_if_database_error_occurs(pool: PgPool) -> Result<(), String> {
        let dao = AnswerDaoImpl::new(pool.clone());
        pool.close().await;

        let some_uuid = "a22abcd2-22ab-2222-a22b-2abc2a2b22cc";
        let err = dao.get_answers(some_uuid.to_owned()).await.unwrap_err();

        if let DBError::Other(_) = err {
            return Ok(());
        }

        Err(format!("Expected Other but got: {}", err))
    }

    #[sqlx::test]
    async fn get_answers_should_succeed(pool: PgPool) -> Result<(), String> {
        let question_dao = QuestionDaoImpl::new(pool.clone());
        let dao = AnswerDaoImpl::new(pool);
        let question = question_dao
            .create_question(Question {
                title: "title".to_owned(),
                description: "quest".to_owned(),
            })
            .await
            .unwrap();

        let answer1 = dao
            .create_answer(Answer {
                question_uuid: question.question_uuid.clone(),
                content: "content".to_owned(),
            })
            .await
            .unwrap();

        let answer2 = dao
            .create_answer(Answer {
                question_uuid: question.question_uuid.clone(),
                content: "content".to_owned(),
            })
            .await
            .unwrap();

        let result = dao.get_answers(question.question_uuid).await.unwrap();

        assert_eq!(result, vec![answer1, answer2]);
        Ok(())
    }
}
