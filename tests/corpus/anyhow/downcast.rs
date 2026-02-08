//! Test: downcast_ref and downcast methods
//!
//! Iron must transpile downcasting correctly.

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

    pub fn downcast_ref<T: std::error::Error + 'static>(&self) -> Option<&T> {
        self.source
            .as_ref()
            .and_then(|s| s.as_ref().downcast_ref::<T>())
    }

    pub fn downcast<T: std::error::Error + 'static>(self) -> Result<T, Self> {
        if let Some(source) = self.source.take() {
            source.downcast::<T>().map_err(|s| Error {
                msg: self.msg,
                source: Some(s),
            })
        } else {
            Err(self)
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

pub struct CustomError {
    code: i32,
    msg: String,
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomError {}: {}", self.code, self.msg)
    }
}

impl std::error::Error for CustomError {}

pub fn try_downcast_ref(e: &Error) -> Option<&CustomError> {
    e.downcast_ref::<CustomError>()
}

pub fn try_downcast(e: Error) -> Result<CustomError, Error> {
    e.downcast::<CustomError>()
}

pub fn match_on_downcast(e: &Error) -> String {
    if let Some(custom) = e.downcast_ref::<CustomError>() {
        format!("found CustomError: {}", custom)
    } else {
        format!("found other error type")
    }
}
