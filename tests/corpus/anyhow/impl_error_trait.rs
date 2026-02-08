//! Test: Multiple trait bounds with impl Trait
//!
//! Iron must transpile impl Trait with multiple trait bounds.

pub struct Error {
    msg: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    pub fn msg(msg: &str) -> Self {
        Error {
            msg: msg.to_string(),
            source: None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}

impl<E> From<E> for Error
where
    E: std::fmt::Display + std::error::Error + 'static,
{
    fn from(err: E) -> Self {
        Error {
            msg: err.to_string(),
            source: None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn create_displayable_error() -> impl std::fmt::Display + std::error::Error {
    Error::msg("displayable error")
}

pub fn wrap_as_error<E>(err: E) -> impl std::fmt::Display + std::error::Error
where
    E: std::fmt::Display + std::error::Error + 'static,
{
    Error::from(err)
}

pub fn format_error_pair(
    e1: &(impl std::fmt::Display + std::error::Error),
    e2: &(impl std::fmt::Display + std::error::Error),
) -> String {
    format!("{}; {}", e1, e2)
}

pub fn collect_errors<E>(errs: Vec<E>) -> Vec<impl std::fmt::Display + std::error::Error>
where
    E: std::fmt::Display + std::error::Error + 'static,
{
    errs.into_iter().map(|e| Error::from(e)).collect()
}
