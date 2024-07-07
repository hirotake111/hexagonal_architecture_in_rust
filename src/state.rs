use std::sync::Arc;

use crate::repository::AuthorRepository;

#[derive(Debug, Clone)]
/// The application state available to all request handlers.
pub struct AppState<AR: AuthorRepository> {
    pub author_repo: Arc<AR>,
}
