use std::future::Future;

use crate::{
    domain::{Author, CreateAuthorError, CreateAuthorRequest},
    repository::AuthorRepository,
};
/// `AuthorMetrics` describes an aggregator of author-related metrics, such as a time-series
/// database
pub trait AuthorMetrics: Send + Sync + Clone + 'static {
    /// Record a successful author creation
    fn record_creation_success(&self) -> impl Future<Output = ()> + Send;

    /// Record an author creation failure
    fn record_creation_failure(&self) -> impl Future<Output = ()> + Send;
}

#[derive(Debug, Clone)]
pub struct Prometheus;
impl AuthorMetrics for Prometheus {
    async fn record_creation_success(&self) {}

    async fn record_creation_failure(&self) {}
}

/// `AuthorNotifier` triggers notification to authors
pub trait AuthorNotifier: Send + Sync + Clone + 'static {
    fn author_created(&self, author: &Author) -> impl Future<Output = ()> + Send;
}

#[derive(Debug, Clone)]
pub struct EmailClient;
impl AuthorNotifier for EmailClient {
    async fn author_created(&self, _author: &Author) {}
}

pub trait AuthorService: Clone + Send + Sync + 'static {
    /// Asynchronously create a new [Author].
    ///
    /// ## Errors
    ///
    /// - [CreateAuthorError:Duplicate] if an [Author] with the same name [AuthorName] already
    /// exists.
    fn create_author(
        &self,
        req: &CreateAuthorRequest,
    ) -> impl Future<Output = Result<Author, CreateAuthorError>> + Send;
}

/// Canonical implementation of the [AuthorService] port, through which the author domain API is
/// consumed
#[derive(Debug, Clone)]
pub struct Service<R, M, N>
where
    R: AuthorRepository,
    M: AuthorMetrics,
    N: AuthorNotifier,
{
    repo: R,
    metrics: M,
    notifier: N,
}

impl<R, M, N> Service<R, M, N>
where
    R: AuthorRepository,
    M: AuthorMetrics,
    N: AuthorNotifier,
{
    pub fn new(r: R, m: M, n: N) -> Self {
        Self {
            repo: r,
            metrics: m,
            notifier: n,
        }
    }
}

impl<R, M, N> AuthorService for Service<R, M, N>
where
    R: AuthorRepository,
    M: AuthorMetrics,
    N: AuthorNotifier,
{
    async fn create_author(&self, req: &CreateAuthorRequest) -> Result<Author, CreateAuthorError> {
        let result = self.repo.create_author(req).await;
        if result.is_err() {
            self.metrics.record_creation_failure().await;
        } else {
            self.metrics.record_creation_success().await;
            self.notifier.author_created(result.as_ref().unwrap()).await;
        }
        result
    }
}

#[cfg(test)]
mod tests {

    use anyhow::anyhow;
    use uuid::Uuid;

    use crate::{
        domain::{Author, AuthorName},
        service::*,
    };

    #[derive(Debug, Clone)]
    struct MockRepository {
        value: Option<Author>,
    }
    impl MockRepository {
        fn new(value: Option<Author>) -> Self {
            Self { value }
        }
        fn get(&self) -> Result<Author, CreateAuthorError> {
            if let Some(author) = &self.value {
                Ok(author.clone())
            } else {
                Err(CreateAuthorError::Unknown(anyhow!("asdf")))
            }
        }
    }

    #[derive(Debug, Clone)]
    struct MockMetrics {}
    impl AuthorMetrics for MockMetrics {
        async fn record_creation_success(&self) {}

        async fn record_creation_failure(&self) {}
    }

    impl AuthorRepository for MockRepository {
        async fn create_author(
            &self,
            _req: &CreateAuthorRequest,
        ) -> Result<Author, CreateAuthorError> {
            self.get()
        }
    }

    #[derive(Debug, Clone)]
    struct MockNotifier {}
    impl AuthorNotifier for MockNotifier {
        async fn author_created(&self, _author: &Author) {}
    }

    #[tokio::test]
    async fn test_create_author() {
        let id = Uuid::new_v4();
        let author = Author::new(id, AuthorName::new("alice").unwrap());
        let repo = MockRepository::new(Some(author.clone()));
        let metrics = MockMetrics {};
        let notifier = MockNotifier {};
        let sut = Service::new(repo, metrics, notifier);
        let req = CreateAuthorRequest::new(AuthorName::new("bob").unwrap());
        let result = sut.create_author(&req).await.unwrap();
        assert_eq!(result, author);
    }
}
