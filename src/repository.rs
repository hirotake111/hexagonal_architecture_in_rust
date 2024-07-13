use std::future::Future;

use anyhow::{anyhow, Context};
use sqlx::Executor;
use uuid::Uuid;

use crate::domain;

/// `AuthorRepository` represents a store of author data
pub trait AuthorRepository: Clone + Send + Sync + 'static {
    /// Persist a new [Author]
    ///
    /// ## Errors
    ///
    /// - MUST return [CreateAuthorError::Duplicate] if an [Author] with the same name [AuthorName] already exists.
    fn create_author(
        &self,
        req: &domain::CreateAuthorRequest,
    ) -> impl Future<Output = Result<domain::Author, domain::CreateAuthorError>> + Send;
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

    #[cfg(test)]
    async fn delete_author(&self, req: &domain::DeleteAuthorRequest) -> anyhow::Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to start SQLite transaction")?;
        let query = sqlx::query!("DELETE FROM authors WHERE id = $1", req.id());
        tx.execute(query).await?;
        tx.commit()
            .await
            .context("failed to commit PostgreSQL transaction")?;
        Ok(())
    }
}

impl AuthorRepository for Postgres {
    async fn create_author(
        &self,
        req: &domain::CreateAuthorRequest,
    ) -> Result<domain::Author, domain::CreateAuthorError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to start SQLite transaction")?;
        let author_id = save_author(&mut tx, req.name()).await.map_err(|e| {
            if is_unique_constraint_violation(&e) {
                domain::CreateAuthorError::Duplicate {
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
        Ok(domain::Author::new(author_id, req.name().clone()))
    }
}

const POSTGRES_UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "23505";

fn is_unique_constraint_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            return code == POSTGRES_UNIQUE_CONSTRAINT_VIOLATION_CODE;
        }
    }
    false
}

async fn save_author(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    name: &domain::AuthorName,
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

#[cfg(test)]
mod tests {

    use crate::domain::{AuthorName, CreateAuthorRequest, DeleteAuthorRequest};

    use super::{AuthorRepository, Postgres};

    #[tokio::test]
    async fn test_create_author() -> anyhow::Result<()> {
        let sut = Postgres::new("postgres://postgres:supersecret@localhost:5432/postgres").await?;
        let author_name = AuthorName::new("alice")?;
        let req = CreateAuthorRequest::new(author_name);
        let result = sut.create_author(&req).await?;
        assert_eq!(result.name(), &AuthorName::new("alice")?);
        let req = DeleteAuthorRequest::new(*result.id());
        sut.delete_author(&req).await?;
        Ok(())
    }
}
