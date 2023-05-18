use async_trait::async_trait;
use sqlx::{types::Uuid, PgPool};

use crate::models::{DBError, Question, QuestionDetail};

#[async_trait]
pub trait QuestionDao {
    async fn create_question(&self, question: Question) -> Result<QuestionDetail, DBError>;
    async fn delete_question(&self, question_uuid: String) -> Result<(), DBError>;
    async fn get_questions(&self) -> Result<Vec<QuestionDetail>, DBError>;
}

pub struct QuestionDaoImpl {
    db: PgPool,
}

impl QuestionDaoImpl {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl QuestionDao for QuestionDaoImpl {
    async fn create_question(&self, question: Question) -> Result<QuestionDetail, DBError> {
        let result = sqlx::query!(
            r#"
                INSERT INTO questions ( title, description )
                VALUES ( $1, $2 )
                RETURNING *
            "#,
            &question.title,
            &question.description
        )
        .fetch_one(&self.db)
        .await;

        let Ok(result) = result else {
            return Err(DBError::Other(Box::new(result.err().unwrap())));
        };

        Ok(QuestionDetail {
            question_uuid: result.question_uuid.to_string(),
            title: result.title,
            description: result.description,
            created_at: result.created_at.to_string(),
        })
    }

    async fn delete_question(&self, question_uuid: String) -> Result<(), DBError> {
        let question_uuid = Uuid::parse_str(&question_uuid)
            .map_err(|error| DBError::InvalidUUID(error.to_string()))?;

        let result = sqlx::query!(
            r#"
                DELETE from questions
                WHERE question_uuid = $1
            "#,
            question_uuid,
        )
        .execute(&self.db)
        .await;

        if let Err(err) = result {
            return Err(DBError::Other(Box::new(err)));
        }

        Ok(())
    }

    async fn get_questions(&self) -> Result<Vec<QuestionDetail>, DBError> {
        let result = sqlx::query!(
            r#"
                SELECT question_uuid, title, description, created_at
                FROM questions
            "#
        )
        .fetch_all(&self.db)
        .await;

        match result {
            Ok(result) => {
                let questions = result
                    .iter()
                    .map(|val| QuestionDetail {
                        question_uuid: val.question_uuid.to_string(),
                        title: val.title.clone(),
                        description: val.description.clone(),
                        created_at: val.created_at.to_string(),
                    })
                    .collect();
                Ok(questions)
            }
            Err(e) => Err(DBError::Other(Box::new(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DBError, Question};
    use sqlx::PgPool;

    #[sqlx::test]
    async fn create_question_should_fail_if_database_error_occours(
        pool: PgPool,
    ) -> Result<(), String> {
        let dao = QuestionDaoImpl::new(pool.clone());
        pool.close().await;

        let result = dao
            .create_question(Question {
                title: "some_title".to_owned(),
                description: "some_desc".to_owned(),
            })
            .await;

        if result.is_ok() {
            return Err(format!("Expected and error but got: {:?} ", result.ok()));
        }

        if let Err(DBError::Other(_)) = result {
            return Ok(());
        }

        return Err(format!("Expected other error but got: {:?} ", result.err()));
    }

    #[sqlx::test]
    async fn create_question_should_success(pool: PgPool) -> Result<(), String> {
        let dao = QuestionDaoImpl::new(pool.clone());

        let result = dao
            .create_question(Question {
                title: "some_title".to_owned(),
                description: "some_desc".to_owned(),
            })
            .await
            .map_err(|e| format!("An not expected error ocourred: {:?}", e))?;

        assert_eq!(result.title, "some_title".to_owned());
        assert_eq!(result.description, "some_desc".to_owned());

        pool.close().await;
        Ok(())
    }

    #[sqlx::test]
    async fn delete_question_should_fail_on_malformed_uuid(pool: PgPool) -> Result<(), String> {
        let dao = QuestionDaoImpl::new(pool);
        let err = dao
            .delete_question("invalid_uui".to_owned())
            .await
            .unwrap_err();

        if let DBError::InvalidUUID(_) = err {
            return Ok(());
        }

        Err(format!("Expected InvalidUUID but got: {}", err))
    }

    #[sqlx::test]
    async fn delete_question_should_fail_if_database_error_occours(
        pool: PgPool,
    ) -> Result<(), String> {
        let dao = QuestionDaoImpl::new(pool.clone());
        pool.close().await;
        let some_uuid = "a22abcd2-22ab-2222-a22b-2abc2a2b22cc";
        let err = dao.delete_question(some_uuid.to_owned()).await.unwrap_err();

        match err {
            DBError::Other(_) => Ok(()),
            err => Err(format!("Expected other but got: {}", err)),
        }
    }

    #[sqlx::test]
    async fn delete_question_should_succeed(pool: PgPool) -> Result<(), String> {
        let valid_uuid = "a22abcd2-22ab-2222-a22b-2abc2a2b22cc";
        let dao = QuestionDaoImpl::new(pool);
        let result = dao
            .delete_question(valid_uuid.to_owned())
            .await
            .map_err(|err| format!("Expected Ok but got: {}", err))?;

        assert_eq!(result, ());
        Ok(())
    }

    #[sqlx::test]
    async fn get_questions_should_fail_on_database_error(pool: PgPool) -> Result<(), String> {
        let dao = QuestionDaoImpl::new(pool.clone());
        pool.close().await;
        let err = dao.get_questions().await.unwrap_err();

        match err {
            DBError::Other(_) => Ok(()),
            err => Err(format!("Expected other but got: {}", err)),
        }
    }

    #[sqlx::test]
    async fn get_questions_should_succeed(pool: PgPool) -> Result<(), String> {
        let dao = QuestionDaoImpl::new(pool.clone());
        let question1 = dao
            .create_question(Question {
                title: "some_title".to_owned(),
                description: "some_desc".to_owned(),
            })
            .await
            .unwrap();
        let question2 = dao
            .create_question(Question {
                title: "some_title".to_owned(),
                description: "some_desc".to_owned(),
            })
            .await
            .unwrap();

        let result = dao
            .get_questions()
            .await
            .map_err(|e| format!("Expected Ok but got: {}", e))?;

        assert_eq!(result, vec![question1, question2]);
        Ok(())
    }
}
