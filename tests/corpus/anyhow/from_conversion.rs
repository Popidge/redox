//! Test: impl From<T> for Error
//!
//! Iron must transpile generic From implementations correctly.

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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::msg(&err.to_string())
    }
}

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

pub fn read_file(path: &str) -> Result<String> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) => Err(Error::from(e)),
    }
}

pub fn create_io_error(msg: &str) -> Error {
    Error::from(std::io::Error::new(std::io::ErrorKind::Other, msg))
}
