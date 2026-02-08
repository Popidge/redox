//! Test: Trait object syntax in function signatures
//!
//! Iron must transpile function signatures with trait objects correctly.

pub struct Error {
    msg: String,
}

pub struct CustomError {
    code: i32,
}

pub fn create_error() -> Error {
    Error {
        msg: "test error".to_string(),
    }
}

pub fn get_message(_e: &dyn std::error::Error) -> String {
    "test".to_string()
}

pub fn get_source(_e: &Error) -> Option<&(dyn std::error::Error + 'static)> {
    None
}

pub fn downcast_custom(_e: &dyn std::error::Error) -> Option<&CustomError> {
    None
}

pub fn wrap_boxed(_err: &dyn std::error::Error) -> Error {
    Error {
        msg: "wrapped".to_string(),
    }
}
