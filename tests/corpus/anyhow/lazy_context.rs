//! Test: with_context(|| ...) - Closures in method arguments
//!
//! Iron must transpile closures passed as method arguments correctly.

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

pub type Result<T> = std::result::Result<T, Error>;

pub trait WithContext<T, E> {
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> WithContext<T, E> for std::result::Result<T, E>
where
    E: std::error::Error,
{
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|err| Error {
            msg: f().to_string(),
            source: Some(Box::new(err)),
        })
    }
}

pub fn read_with_lazy_context(path: &str) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("failed to read file at path: {}", path))
}

pub fn parse_with_lazy_context(s: &str) -> Result<i32> {
    s.trim()
        .parse::<i32>()
        .with_context(|| format!("could not parse '{}' as integer", s))
}

pub fn multiple_lazy_contexts() -> Result<i32> {
    let a_str = std::fs::read_to_string("a.txt")?;
    let a: i32 = a_str
        .trim()
        .parse::<i32>()
        .with_context(|| "failed to parse a.txt".to_string())?;

    let b_str = std::fs::read_to_string("b.txt")?;
    let b: i32 = b_str
        .trim()
        .parse::<i32>()
        .with_context(|| "failed to parse b.txt".to_string())?;

    Ok(a + b)
}
