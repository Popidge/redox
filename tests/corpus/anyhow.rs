//! Combined anyhow corpus for roundtrip testing
//!
//! This module combines all anyhow pattern tests into a single file
//! for easier roundtrip testing through Iron.

#[path = "anyhow/mod.rs"]
mod anyhow;

#[path = "anyhow/box_dyn_error.rs"]
mod box_dyn_error;

#[path = "anyhow/impl_iterator.rs"]
mod impl_iterator;

#[path = "anyhow/impl_trait.rs"]
mod impl_trait;

#[path = "anyhow/downcast.rs"]
mod downcast;

#[path = "anyhow/from_conversion.rs"]
mod from_conversion;

#[path = "anyhow/context_method.rs"]
mod context_method;

#[path = "anyhow/source_chaining.rs"]
mod source_chaining;

#[path = "anyhow/result_alias.rs"]
mod result_alias;

#[path = "anyhow/lazy_context.rs"]
mod lazy_context;

#[path = "anyhow/impl_error_trait.rs"]
mod impl_error_trait;
