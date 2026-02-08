//! Rust to Iron mapping definitions
//!
//! This module contains the dictionaries and transformation rules for converting
//! Rust AST constructs to Iron syntax.

use crate::keywords::sanitize_identifier;
use syn::{FnArg, Pat, PatType, ReturnType, Type};

/// Maps Rust types to Iron type representations
pub fn map_type_to_iron(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            if let Some(segment) = path.segments.last() {
                let name = segment.ident.to_string();

                // Handle generics
                match &segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        let generic_args: Vec<String> = args
                            .args
                            .iter()
                            .map(|arg| match arg {
                                syn::GenericArgument::Type(t) => map_type_to_iron(t),
                                _ => "unknown".to_string(),
                            })
                            .collect();

                        map_generic_type(&name, &generic_args)
                    }
                    _ => map_simple_type(&name),
                }
            } else {
                "unknown".to_string()
            }
        }
        Type::Reference(type_ref) => {
            let inner = map_type_to_iron(&type_ref.elem);
            if type_ref.mutability.is_some() {
                format!("mutable reference to {}", inner)
            } else {
                format!("reference to {}", inner)
            }
        }
        Type::Ptr(type_ptr) => {
            let inner = map_type_to_iron(&type_ptr.elem);
            if type_ptr.mutability.is_some() {
                format!("mutable raw pointer to {}", inner)
            } else {
                format!("raw pointer to {}", inner)
            }
        }
        Type::Tuple(tuple) => {
            if tuple.elems.is_empty() {
                "unit".to_string()
            } else {
                let types: Vec<String> = tuple.elems.iter().map(map_type_to_iron).collect();
                format!("tuple of {}", types.join(" and "))
            }
        }
        Type::Array(array) => {
            let inner = map_type_to_iron(&array.elem);
            format!("array of {}", inner)
        }
        Type::Slice(slice) => {
            let inner = map_type_to_iron(&slice.elem);
            format!("slice of {}", inner)
        }
        Type::BareFn(fn_type) => {
            let inputs: Vec<String> = fn_type
                .inputs
                .iter()
                .map(|arg| map_type_to_iron(&arg.ty))
                .collect();
            let output = match &fn_type.output {
                ReturnType::Default => "unit".to_string(),
                ReturnType::Type(_, ty) => map_type_to_iron(ty),
            };
            format!(
                "function taking {} returning {}",
                inputs.join(" and "),
                output
            )
        }
        Type::Paren(paren_type) => map_type_to_iron(&paren_type.elem),
        Type::TraitObject(type_trait) => type_trait
            .bounds
            .iter()
            .filter_map(|bound| {
                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                    trait_bound
                        .path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_else(|| "unknown_type".to_string()),
        _ => "unknown_type".to_string(),
    }
}

fn map_simple_type(name: &str) -> String {
    match name {
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => name.to_string(),
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => name.to_string(),
        "f32" | "f64" => name.to_string(),
        "bool" => "boolean".to_string(),
        "char" => "character".to_string(),
        "str" => "string slice".to_string(),
        "String" => "string".to_string(),
        "Vec" => "list".to_string(),
        "Box" => "box".to_string(),
        "Option" => "optional".to_string(),
        "Result" => "result".to_string(),
        "HashMap" => "hash map".to_string(),
        "Rc" => "reference counted".to_string(),
        "Arc" => "atomic reference counted".to_string(),
        _ => sanitize_identifier(name),
    }
}

fn map_generic_type(name: &str, args: &[String]) -> String {
    match name {
        "Vec" => format!("list of {}", args.join(", ")),
        "Box" => format!("box containing {}", args.join(", ")),
        "Option" => format!("optional {}", args.join(", ")),
        "Result" => {
            if args.len() >= 2 {
                format!("result of {} or error {}", args[0], args[1])
            } else {
                let sanitized = sanitize_identifier(name);
                format!("{} of {}", sanitized, args.join(" and "))
            }
        }
        "HashMap" => {
            if args.len() >= 2 {
                format!("hash map from {} to {}", args[0], args[1])
            } else {
                format!("hash map {}", args.join(", "))
            }
        }
        "Rc" => format!("reference counted {}", args.join(", ")),
        "Arc" => format!("atomic reference counted {}", args.join(", ")),
        _ => {
            let sanitized = sanitize_identifier(name);
            format!("{} of {}", sanitized, args.join(", "))
        }
    }
}

/// Maps Rust return types to Iron return type representations
pub fn map_return_type(return_type: &ReturnType) -> String {
    match return_type {
        ReturnType::Default => "unit".to_string(),
        ReturnType::Type(_, ty) => map_type_to_iron(ty),
    }
}

/// Maps function arguments to Iron parameter representations
pub fn map_fn_arg(arg: &FnArg) -> Option<(String, String)> {
    match arg {
        FnArg::Typed(PatType { pat, ty, .. }) => {
            let name = match &**pat {
                Pat::Ident(pat_ident) => sanitize_identifier(&pat_ident.ident.to_string()),
                _ => "unnamed".to_string(),
            };
            let ty_str = map_type_to_iron(ty);
            Some((name, ty_str))
        }
        FnArg::Receiver(receiver) => {
            if receiver.mutability.is_some() {
                Some((
                    "context".to_string(),
                    "mutable reference to context".to_string(),
                ))
            } else if receiver.reference.is_some() {
                Some(("context".to_string(), "reference to context".to_string()))
            } else {
                Some(("context".to_string(), "context".to_string()))
            }
        }
    }
}

/// Maps Rust binary operators to Iron representations
pub fn map_binary_op(op: &syn::BinOp) -> String {
    match op {
        syn::BinOp::Add(_) => "plus".to_string(),
        syn::BinOp::Sub(_) => "minus".to_string(),
        syn::BinOp::Mul(_) => "times".to_string(),
        syn::BinOp::Div(_) => "divided by".to_string(),
        syn::BinOp::Rem(_) => "modulo".to_string(),
        syn::BinOp::And(_) => "and".to_string(),
        syn::BinOp::Or(_) => "or".to_string(),
        syn::BinOp::BitXor(_) => "bitwise xor".to_string(),
        syn::BinOp::BitAnd(_) => "bitwise and".to_string(),
        syn::BinOp::BitOr(_) => "bitwise or".to_string(),
        syn::BinOp::Shl(_) => "shift left".to_string(),
        syn::BinOp::Shr(_) => "shift right".to_string(),
        syn::BinOp::Eq(_) => "equal to".to_string(),
        syn::BinOp::Lt(_) => "less than".to_string(),
        syn::BinOp::Le(_) => "less than or equal to".to_string(),
        syn::BinOp::Ne(_) => "not equal to".to_string(),
        syn::BinOp::Ge(_) => "greater than or equal to".to_string(),
        syn::BinOp::Gt(_) => "greater than".to_string(),
        syn::BinOp::AddAssign(_) => "plus equals".to_string(),
        syn::BinOp::SubAssign(_) => "minus equals".to_string(),
        syn::BinOp::MulAssign(_) => "times equals".to_string(),
        syn::BinOp::DivAssign(_) => "divided by equals".to_string(),
        syn::BinOp::RemAssign(_) => "modulo equals".to_string(),
        syn::BinOp::BitXorAssign(_) => "bitwise xor equals".to_string(),
        syn::BinOp::BitAndAssign(_) => "bitwise and equals".to_string(),
        syn::BinOp::BitOrAssign(_) => "bitwise or equals".to_string(),
        syn::BinOp::ShlAssign(_) => "shift left equals".to_string(),
        syn::BinOp::ShrAssign(_) => "shift right equals".to_string(),
        _ => "unknown operator".to_string(),
    }
}

/// Maps Rust unary operators to Iron representations
pub fn map_unary_op(op: &syn::UnOp) -> String {
    match op {
        syn::UnOp::Deref(_) => "dereference".to_string(),
        syn::UnOp::Not(_) => "not".to_string(),
        syn::UnOp::Neg(_) => "negate".to_string(),
        _ => "unknown unary operator".to_string(),
    }
}
