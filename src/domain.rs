use axum::{http::StatusCode, response::IntoResponse};
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

impl AuthorName {
    pub fn val(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AuthorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

#[derive(Debug, Error)]
pub enum CreateAuthorError {
    #[error("author with name {name} already exists")]
    Duplicate { name: AuthorName },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
    // to be extended as new error scenarios are introduced
}

pub struct CreateAuthorRequest {
    name: AuthorName,
}

impl CreateAuthorRequest {
    pub fn new(name: AuthorName) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &AuthorName {
        &self.name
    }
}

pub struct DeleteAuthorRequest {
    id: Uuid,
}

impl DeleteAuthorRequest {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
    pub fn id(&self) -> &Uuid {
        &self.id
    }
}

pub enum ApiError {
    InternalServerError(String),
    UnprocessableEntity(String),
}

impl From<CreateAuthorError> for ApiError {
    fn from(e: CreateAuthorError) -> Self {
        match e {
            CreateAuthorError::Duplicate { name } => {
                Self::UnprocessableEntity(format!("author with name {} already exists", name))
            }
            CreateAuthorError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<AuthorNameEmptyError> for ApiError {
    fn from(_: AuthorNameEmptyError) -> Self {
        ApiError::UnprocessableEntity("author name cannot be empty".to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::UnprocessableEntity(e) => {
                (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            }
            ApiError::InternalServerError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        }
    }
}
