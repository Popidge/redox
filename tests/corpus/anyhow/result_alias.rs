//! Test: Type alias for Result<T, Error>
//!
//! Iron must transpile type aliases with generic defaults correctly.

pub struct Error {
    msg: String,
}

impl Error {
    pub fn msg(msg: &str) -> Self {
        Error {
            msg: msg.to_string(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}

pub type AnyhowResult<T> = std::result::Result<T, Error>;

pub type IoResult<T> = std::io::Result<T>;

pub fn simple_alias_return() -> AnyhowResult<String> {
    Ok("hello".to_string())
}

pub fn chained_alias() -> AnyhowResult<i32> {
    let a_str = std::fs::read_to_string("a.txt")?;
    let a: i32 = a_str.trim().parse()?;
    let b_str = std::fs::read_to_string("b.txt")?;
    let b: i32 = b_str.trim().parse()?;
    Ok(a + b)
}

pub fn with_io_alias(path: &str) -> IoResult<Vec<u8>> {
    std::fs::read(path)
}
