/// `AuthorRepository` represents a store of author data
pub trait AuthorRepository {
    /// Persist a new [Author]
    ///
    /// ## Errors
    ///
    /// - MUST return [CreateAuthorError::Duplicate] if an [Author] with the same name [AuthorName] already exists.
    fn create_author(&self, req: &CreateAuthorRequest) -> Result<Author, CreateAuthorRequest>;
}
