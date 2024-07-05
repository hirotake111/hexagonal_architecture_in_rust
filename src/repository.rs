use std::str::FromStr;

use anyhow::Context;

use crate::domain::{Author, CreateAuthorError, CreateAuthorRequest};

/// `AuthorRepository` represents a store of author data
pub trait AuthorRepository {
    /// Persist a new [Author]
    ///
    /// ## Errors
    ///
    /// - MUST return [CreateAuthorError::Duplicate] if an [Author] with the same name [AuthorName] already exists.
    fn create_author(&self, req: &CreateAuthorRequest) -> Result<Author, CreateAuthorError>;
}
#[derive(Debug, Clone)]
pub struct Sqlite {
    pool: sqlx::SqlitePool,
}

impl Sqlite {
    pub async fn new(path: &str) -> anyhow::Result<Sqlite> {
        let pool = sqlx::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::from_str(path)
                .with_context(|| format!("invalid database path: {}", path))?
                .pragma("foreign_keys", "ON"),
        )
        .await
        .with_context(|| format!("failed to open database at {}", path))?;

        Ok(Sqlite { pool })
    }
}

impl AuthorRepository for Sqlite {
    async fn create_author(&self, req: &CreateAuthorRequest) -> Result<Author, CreateAuthorError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to start SQLite transaction")?;
        let author_id = self.save_author(&mut tx, req.name())
    }
}
