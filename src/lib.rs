//! Redox - Rust to Iron Transpiler
//!
//! A deterministic source-to-source transpiler that converts Rust code into Iron,
//! a verbose, lexically-expanded superset of Rust designed for optimal tokenization
//! by Large Language Models.

pub mod emitter;
pub mod iron_ast;
pub mod iron_parser;
pub mod iron_tokenizer;
pub mod keywords;
pub mod mappings;
pub mod oxidation;
pub mod parser;

use parser::IronParser;
use syn::File;

/// Error type for transpilation failures
#[derive(Debug, Clone)]
pub enum TranspileError {
    ParseError(String),
    UnsupportedSyntax(String),
    InternalError(String),
}

impl std::fmt::Display for TranspileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranspileError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            TranspileError::UnsupportedSyntax(msg) => {
                write!(f, "Unsupported syntax: {}", msg)
            }
            TranspileError::InternalError(msg) => {
                write!(f, "Internal error: {}", msg)
            }
        }
    }
}

impl std::error::Error for TranspileError {}

/// Transpile Rust source code to Iron
///
/// # Arguments
///
/// * `source` - The Rust source code as a string
///
/// # Returns
///
/// * `Ok(String)` - The Iron code if successful
/// * `Err(TranspileError)` - Error details if transpilation fails
///
/// # Example
///
/// ```rust
/// use redox::transpile;
///
/// let rust_code = r#"
/// fn hello() {
///     println!("Hello, World!");
/// }
/// "#;
///
/// match transpile(rust_code) {
///     Ok(iron_code) => println!("{}", iron_code),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn transpile(source: &str) -> Result<String, TranspileError> {
    // Parse the Rust source
    let file = syn::parse_str::<File>(source).map_err(|e| {
        TranspileError::ParseError(format!(
            "Failed to parse Rust source at {:?}: {}",
            e.span(),
            e
        ))
    })?;

    // Create parser and transpile
    let mut parser = IronParser::new();

    parser
        .parse_file(&file)
        .map_err(|errors| TranspileError::UnsupportedSyntax(errors.join("; ")))
}

/// Transpile a Rust file to Iron
///
/// # Arguments
///
/// * `file` - A parsed syn::File AST
///
/// # Returns
///
/// * `Ok(String)` - The Iron code if successful
/// * `Err(TranspileError)` - Error details if transpilation fails
pub fn transpile_file(file: &File) -> Result<String, TranspileError> {
    let mut parser = IronParser::new();

    parser
        .parse_file(file)
        .map_err(|errors| TranspileError::UnsupportedSyntax(errors.join("; ")))
}

/// Check if Iron code is valid (basic validation)
///
/// This function checks if the generated Iron code contains any
/// prohibited symbols that should have been transformed.
///
/// # Arguments
///
/// * `iron_code` - The Iron code to validate
///
/// # Returns
///
/// `true` if valid, `false` otherwise
pub fn validate_iron(iron_code: &str) -> bool {
    let prohibited_chars = ['&', '-', '>', '<', '*'];

    for ch in prohibited_chars {
        if iron_code.contains(ch) {
            return false;
        }
    }

    // Check for :: pattern (namespace separator)
    if iron_code.contains("::") {
        return false;
    }

    true
}

/// Oxidize Iron code to Rust
///
/// # Arguments
///
/// * `iron_source` - The Iron source code as a string
///
/// # Returns
///
/// * `Ok(String)` - The Rust code if successful
/// * `Err(TranspileError)` - Error details if oxidation fails
///
/// # Example
///
/// ```rust
/// use redox::oxidize;
///
/// let iron_code = r#"
/// function hello
/// begin
///     return 42
/// end function
/// "#;
///
/// match oxidize(iron_code) {
///     Ok(rust_code) => println!("{}", rust_code),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn oxidize(iron_source: &str) -> Result<String, TranspileError> {
    use iron_parser::IronParser;
    use oxidation::Oxidizer;

    // Parse the Iron source
    let mut parser = IronParser::new(iron_source);
    let ast = parser
        .parse()
        .map_err(|e| TranspileError::ParseError(format!("{:?}", e)))?;

    // Convert to Rust
    let mut oxidizer = Oxidizer::new();
    Ok(oxidizer.oxidize(&ast))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile_simple_function() {
        let rust = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

        let result = transpile(rust);
        assert!(result.is_ok());

        let iron = result.unwrap();
        assert!(iron.contains("function"));
        assert!(iron.contains("add"));
        assert!(validate_iron(&iron));
    }

    #[test]
    fn test_validate_iron() {
        assert!(validate_iron("function foo begin end function"));
        assert!(!validate_iron("function &foo begin end function"));
        assert!(!validate_iron("function -> foo begin end function"));
    }

    #[test]
    fn test_transpile_with_generics() {
        let rust = r#"
fn identity<T>(x: T) -> T {
    x
}
"#;

        let result = transpile(rust);
        assert!(result.is_ok());

        let iron = result.unwrap();
        assert!(iron.contains("generic"));
        assert!(iron.contains("type T"));
    }
}
