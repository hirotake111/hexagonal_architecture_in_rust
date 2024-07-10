use std::sync::Arc;

use crate::service::AuthorService;

#[derive(Debug, Clone)]
/// The application state available to all request handlers.
pub struct AppState<AS: AuthorService> {
    pub author_service: Arc<AS>,
}
