use std::future::Future;

use anyhow::{anyhow, Context};
use sqlx::Executor;
use uuid::Uuid;

use crate::domain::{Author, AuthorName, CreateAuthorError, CreateAuthorRequest};

/// `AuthorRepository` represents a store of author data
pub trait AuthorRepository: Clone + Send + Sync + 'static {
    /// Persist a new [Author]
    ///
    /// ## Errors
    ///
    /// - MUST return [CreateAuthorError::Duplicate] if an [Author] with the same name [AuthorName] already exists.
    fn create_author(
        &self,
        req: &CreateAuthorRequest,
    ) -> impl Future<Output = Result<Author, CreateAuthorError>> + Send;
}

#[derive(Debug, Clone)]
pub struct Postgres {
    pool: sqlx::PgPool,
}

impl Postgres {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }
}

impl AuthorRepository for Postgres {
    async fn create_author(&self, req: &CreateAuthorRequest) -> Result<Author, CreateAuthorError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to start SQLite transaction")?;
        let author_id = save_author(&mut tx, req.name()).await.map_err(|e| {
            if is_unique_constraint_violation(&e) {
                CreateAuthorError::Duplicate {
                    name: req.name().clone(),
                }
            } else {
                anyhow!(e)
                    .context(format!("failed to save author with name: {:?}", req.name()))
                    .into()
            }
        })?;

        tx.commit()
            .await
            .context("failed to commit PostgreSQL transaction")?;
        Ok(Author::new(author_id, req.name().clone()))
    }
}

const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "2067";

fn is_unique_constraint_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            return code == UNIQUE_CONSTRAINT_VIOLATION_CODE;
        }
    }
    return false;
}

async fn save_author(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    name: &AuthorName,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    let query = sqlx::query!(
        "INSERT INTO authors (id, name) VALUES ($1, $2)",
        id,
        name.val()
    );
    tx.execute(query).await?;
    Ok(id)
}
