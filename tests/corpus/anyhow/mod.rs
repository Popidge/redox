//! Minimal anyhow simulation for testing Iron's transpilation capabilities.
//!
//! This module provides stub types that simulate anyhow's Error and Context
//! patterns. The goal is to test Iron's ability to transpile Rust language
//! features (Box<dyn Trait>, impl Trait, downcasting, closures in args),
//! not to test actual error handling functionality.

use std::error::Error as StdError;
use std::fmt::{self, Display};

pub struct Error {
    msg: String,
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl Error {
    pub fn msg(msg: &str) -> Self {
        Error {
            msg: msg.to_string(),
            source: None,
        }
    }

    pub fn downcast_ref<T: StdError + 'static>(&self) -> Option<&T> {
        self.source
            .as_ref()
            .and_then(|s| s.as_ref().downcast_ref::<T>())
    }

    pub fn downcast<T: StdError + 'static>(self) -> Result<T, Self> {
        if let Some(source) = self.source.take() {
            source.downcast::<T>().map_err(|s| Error {
                msg: self.msg,
                source: Some(s),
            })
        } else {
            Err(self)
        }
    }

    pub fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|s| s.as_ref())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|s| s.as_ref())
    }
}

impl<T> From<T> for Error
where
    T: StdError + Send + Sync + 'static,
{
    fn from(err: T) -> Self {
        Error {
            msg: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

mod private {
    use super::Error;

    pub trait Sealed {}

    impl<T, E> Sealed for std::result::Result<T, E> where E: StdError {}
}

pub trait Context<T, E>: private::Sealed {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;
}

impl<T, E> Context<T, E> for std::result::Result<T, E>
where
    E: StdError,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|err| Error {
            msg: context.to_string(),
            source: Some(Box::new(err)),
        })
    }
}

pub trait WithContext<T, E>: private::Sealed {
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> WithContext<T, E> for std::result::Result<T, E>
where
    E: StdError,
{
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|err| Error {
            msg: f().to_string(),
            source: Some(Box::new(err)),
        })
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Chain<'a> {
    error: &'a Error,
}

impl<'a> Iterator for Chain<'a> {
    type Item = &'a (dyn StdError + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(source) = self.error.source() {
            Some(source)
        } else {
            None
        }
    }
}

impl Error {
    pub fn chain(&self) -> Chain<'_> {
        Chain { error: self }
    }
}
