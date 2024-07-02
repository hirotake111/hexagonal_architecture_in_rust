use thiserror::Error;
use uuid::Uuid;

/// A uniquely identifiable author of blog posts.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Author {
    id: Uuid,
    name: AuthorName,
}

impl Author {
    pub fn new(id: Uuid, name: AuthorName) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn name(&self) -> &AuthorName {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorName(String);

#[derive(Clone, Debug, Error)]
#[error("author name cannot be empty")]
pub struct AuthorNameEmptyError;

impl AuthorName {
    pub fn new(raw: &str) -> Result<Self, AuthorNameEmptyError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            Err(AuthorNameEmptyError)
        } else {
            Ok(AuthorName(trimmed.to_string()))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateAuthorRequest {
    name: AuthorName,
}

impl CreateAuthorRequest {}

#[derive(Debug, Error)]
pub enum CreateAuthorError {
    #[error("author with name {name} already exists")]
    Duplicate { name: AuthorName },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    // to be extended as new error scenarios are introduced
}
