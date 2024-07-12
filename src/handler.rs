use crate::{
    domain::{ApiError, Author, AuthorName, AuthorNameEmptyError, CreateAuthorRequest},
    service::AuthorService,
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiSuccess<T>(axum::http::StatusCode, T);
impl<T> ApiSuccess<T>
where
    T: Serialize,
{
    pub fn new(code: StatusCode, data: T) -> Self {
        Self(code, data)
    }
}

impl<T> IntoResponse for ApiSuccess<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        if let Ok(body) = serde_json::to_string(&self.1) {
            if let Ok(response) = axum::response::Response::builder()
                .status(self.0)
                .header("Content-Type", "application/json")
                .body(body)
            {
                return response.into_response();
            }
        }
        (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
    }
}

pub async fn crate_author<AS: AuthorService>(
    State(state): State<AppState<AS>>,
    Json(body): Json<CreateAuthorHttpRequestBody>,
) -> Result<ApiSuccess<CreateAuthorResponseData>, ApiError> {
    let domain_req = body.try_into_domain()?;
    state
        .author_service
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
    use std::ops::DerefMut;
    use std::{mem, sync::Arc};
    use tokio::sync::Mutex;

    use crate::{
        domain::{Author, CreateAuthorError, CreateAuthorRequest},
        repository::AuthorRepository,
    };

    #[derive(Clone)]
    struct MockAuthorRepository {
        create_author_result: Arc<Mutex<Result<Author, CreateAuthorError>>>,
    }

    impl AuthorRepository for MockAuthorRepository {
        async fn create_author(
            &self,
            _: &CreateAuthorRequest,
        ) -> Result<Author, CreateAuthorError> {
            let mut guard = self.create_author_result.lock().await;
            let mut result = Err(CreateAuthorError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }
    }
}
