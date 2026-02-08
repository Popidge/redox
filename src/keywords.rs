//! Iron keyword protection and collision detection
//!
//! This module defines all reserved Iron keywords and handles name collision
//! detection/resolution when Rust identifiers conflict with Iron primitives.

/// All reserved Iron keywords that cannot be used as identifiers
pub const RESERVED_KEYWORDS: &[&str] = &[
    // Types and references
    "type",
    "reference",
    "mutable",
    "raw",
    "pointer",
    "optional",
    "result",
    "list",
    "box",
    // Control flow
    "if",
    "condition",
    "then",
    "otherwise",
    "compare",
    "case",
    "while",
    "repeat",
    "for",
    "each",
    "in",
    "iterator",
    "loop",
    "forever",
    "exit",
    "continue",
    "return",
    // Functions
    "function",
    "with",
    "generic",
    "implementing",
    "takes",
    "parameter",
    "parameters",
    "returns",
    "begin",
    "end",
    "call",
    "method",
    "on",
    "associated",
    // Bindings
    "define",
    "as",
    "set",
    "equal",
    "to",
    "constant",
    "static",
    // Structs and enums
    "structure",
    "fields",
    "field",
    "enumeration",
    "variants",
    "variant",
    "of",
    // Special values
    "context",
    "self",
    // Note: some, none, ok, error are Iron keywords but we don't reserve them
    // because they map from Rust enum variants (Some, None, Ok, Err)
    // and we handle those specially in the mappings
    // Comments
    "note",
    "that",
];

/// Standard library enum variants that should NOT be sanitized
/// These are handled specially in the transpiler
pub const STANDARD_VARIANTS: &[&str] = &["Some", "None", "Ok", "Err"];

/// Prefix used when a Rust identifier conflicts with Iron keywords
pub const COLLISION_PREFIX: &str = "user_";

/// Check if a name conflicts with Iron reserved keywords
pub fn is_reserved(name: &str) -> bool {
    RESERVED_KEYWORDS.contains(&name.to_lowercase().as_str())
}

/// Transform a Rust identifier to avoid Iron keyword collisions
pub fn sanitize_identifier(name: &str) -> String {
    // Don't sanitize standard library enum variants
    if STANDARD_VARIANTS.contains(&name) {
        return name.to_string();
    }

    if is_reserved(name) {
        format!("{}{}", COLLISION_PREFIX, name)
    } else {
        name.to_string()
    }
}

/// Check if a name is a standard library variant
pub fn is_standard_variant(name: &str) -> bool {
    STANDARD_VARIANTS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reserved_keywords() {
        assert!(is_reserved("function"));
        assert!(is_reserved("define"));
        assert!(is_reserved("reference"));
        assert!(!is_reserved("my_var"));
    }

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("function"), "user_function");
        assert_eq!(sanitize_identifier("my_var"), "my_var");
    }
}
