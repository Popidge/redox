//! Test: impl Trait with Error bounds
//!
//! Iron must transpile impl Trait with trait bounds correctly.

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
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        Error {
            msg: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn create_error() -> impl std::error::Error {
    Error::msg("test error")
}

pub fn wrap_error<E>(err: E) -> impl std::error::Error
where
    E: std::error::Error + 'static,
{
    Error::from(err)
}

pub fn error_or_default<E>(err: Option<E>) -> impl std::error::Error
where
    E: std::error::Error + Default,
{
    match err {
        Some(e) => Error::from(e),
        None => Error::from(E::default()),
    }
}
