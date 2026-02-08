//! Test: .context() method chaining on Result
//!
//! Iron must transpile method calls on Result types correctly.

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

pub trait Context<T, E> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static;
}

impl<T, E> Context<T, E> for std::result::Result<T, E>
where
    E: std::error::Error,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| Error {
            msg: context.to_string(),
            source: Some(Box::new(err)),
        })
    }
}

pub fn add_context_to_result() -> Result<String> {
    std::fs::read_to_string("test.txt").context("could not read test.txt")
}

pub fn chain_context() -> Result<i32> {
    let content = std::fs::read_to_string("number.txt")?;
    let num: i32 = content.trim().parse()?;
    Ok(num)
}

pub fn context_after_map() -> Result<String> {
    let content = std::fs::read_to_string("data.json")?;
    Ok(content.trim().to_string())
}

pub fn nested_context() -> Result<Vec<u8>> {
    let content = std::fs::read("file.bin")?;
    Ok(content)
}
