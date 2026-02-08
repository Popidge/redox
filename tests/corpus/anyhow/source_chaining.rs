//! Test: .source() method for error chaining
//!
//! Iron must transpile trait method calls that return Option<&dyn Trait>.

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

    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|s| s.as_ref())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub type StdError = dyn std::error::Error + 'static;

pub fn get_root_cause(e: &Error) -> Option<&StdError> {
    let mut current = e.source();
    while let Some(source) = current {
        if source.source().is_none() {
            return Some(source);
        }
        current = source.source();
    }
    None
}

pub fn count_error_sources(e: &Error) -> usize {
    let mut count = 0;
    let mut current = e.source();
    while let Some(source) = current {
        count += 1;
        current = source.source();
    }
    count
}

pub fn find_io_source(e: &Error) -> bool {
    let mut current = e.source();
    while let Some(source) = current {
        if let Some(_) = source.downcast_ref::<std::io::Error>() {
            return true;
        }
        current = source.source();
    }
    false
}
