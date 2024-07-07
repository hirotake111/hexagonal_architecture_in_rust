use crate::{
    domain::{ApiError, Author, AuthorName, AuthorNameEmptyError, CreateAuthorRequest},
    repository::AuthorRepository,
    state::AppState,
};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// The body of an [Author] create request
pub struct CreateAuthorHttpRequestBody {
    name: String,
}

impl CreateAuthorHttpRequestBody {
    /// Converts the HTTP request body into a domain request
    fn try_into_domain(self) -> Result<CreateAuthorRequest, AuthorNameEmptyError> {
        let author_name = AuthorName::new(&self.name)?;
        Ok(CreateAuthorRequest::new(author_name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateAuthorResponseData {
    id: String,
}

impl From<&Author> for CreateAuthorResponseData {
    fn from(author: &Author) -> Self {
        Self {
            id: author.id().to_string(),
        }
    }
}

pub struct ApiSuccess<T>((axum::http::StatusCode, T));
impl<T> ApiSuccess<T> {
    pub fn new(code: StatusCode, data: T) -> Self {
        Self((code, data))
    }
}

pub async fn crate_author<AR: AuthorRepository>(
    State(state): State<AppState<AR>>,
    Json(body): Json<CreateAuthorHttpRequestBody>,
) -> Result<ApiSuccess<CreateAuthorResponseData>, ApiError> {
    let domain_req = body.try_into_domain()?;
    state
        .author_repo
        .create_author(&domain_req)
        .await
        .map_err(ApiError::from)
        .map(|ref author| {
            ApiSuccess::<CreateAuthorResponseData>::new(StatusCode::CREATED, author.into())
        })
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use std::{mem, sync::Arc};
    use tokio::sync::Mutex;

    use crate::{
        domain::{Author, CreateAuthorError},
        repository::AuthorRepository,
    };

    #[derive(Clone)]
    struct MockAuthorRepository {
        create_author_result: Arc<Mutex<Result<Author, CreateAuthorError>>>,
    }

    impl AuthorRepository for MockAuthorRepository {
        fn create_author(
            &self,
            req: &crate::domain::CreateAuthorRequest,
        ) -> impl std::future::Future<Output = Result<Author, CreateAuthorError>> + Send {
            let mut guard = self.create_author_result.lock().await;
            let mut result = Err(CreateAuthorError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }
    }
}
