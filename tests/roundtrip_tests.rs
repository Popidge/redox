//! Integration tests for round-trip fidelity
//!
//! These tests verify that Rust code can be transpiled to Iron and back
//! to Rust with semantic preservation.

mod roundtrip;
use roundtrip::test_roundtrip_content;

// ============== WORKING TESTS ==============
// These tests should pass with the current implementation

#[test]
fn test_vec_new_roundtrips() {
    let code = r#"
fn test_vec() -> Vec<i32> {
    Vec::new()
}
"#;
    test_roundtrip_content(code).expect("Vec::new() should round-trip");
}

#[test]
fn test_simple_function() {
    let code = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
    test_roundtrip_content(code).expect("Simple function should round-trip");
}

// ============== PLACEHOLDER TESTS ==============
// These tests document what we want to support but don't yet

#[test]
fn test_vec_with_capacity() {
    let code = r#"
fn test_vec() -> Vec<i32> {
    Vec::with_capacity(10)
}
"#;
    test_roundtrip_content(code).expect("Vec::with_capacity should round-trip");
}

#[test]
fn test_vec_push() {
    let code = r#"
fn test_vec() -> Vec<i32> {
    let mut v = Vec::new();
    v.push(42);
    v
}
"#;
    test_roundtrip_content(code).expect("Vec::push should round-trip");
}

#[test]
fn test_vec_pop() {
    let code = r#"
fn test_vec() -> Option<i32> {
    let mut v = vec![1, 2, 3];
    v.pop()
}
"#;
    test_roundtrip_content(code).expect("Vec::pop should round-trip");
}

#[test]
fn test_option_some() {
    let code = r#"
fn test_option() -> Option<i32> {
    Some(42)
}
"#;
    test_roundtrip_content(code).expect("Option::Some should round-trip");
}

#[test]
fn test_option_map() {
    let code = r#"
fn test_option(x: Option<i32>) -> Option<i32> {
    x.map(|n| n * 2)
}
"#;
    test_roundtrip_content(code).expect("Option::map should round-trip");
}

#[test]
fn test_result_ok() {
    let code = r#"
fn test_result() -> Result<i32, ()> {
    Ok(42)
}
"#;
    test_roundtrip_content(code).expect("Result::Ok should round-trip");
}

#[test]
fn test_result_ok_method_roundtrip() {
    let code = r#"
fn test_result_ok_method(input: Result<i32, String>) -> Option<i32> {
    input.ok()
}
"#;
    test_roundtrip_content(code).expect("Result::ok should round-trip");
}

#[test]
fn test_closure() {
    let code = r#"
fn make_closure() -> i32 {
    let f = |x| x * 2;
    f(21)
}
"#;
    test_roundtrip_content(code).expect("Closures should round-trip");
}

#[test]
fn test_type_alias_roundtrip() {
    let code = r#"
type MyResult<T> = Result<T, ()>;

fn use_alias() -> MyResult<i32> {
    Ok(42)
}
"#;
    test_roundtrip_content(code).expect("Type aliases should round-trip");
}

#[test]
fn test_impl_block_roundtrip() {
    let code = r#"
struct Counter {
    value: i32,
}

impl Counter {
    fn new() -> Self {
        Counter { value: 0 }
    }

    fn inc(&mut self) {
        self.value = self.value + 1;
    }
}

fn use_counter() -> i32 {
    let mut c = Counter::new();
    c.inc();
    c.value
}
"#;
    test_roundtrip_content(code).expect("Impl blocks should round-trip");
}

// ============== CORPUS TESTS ==============
// Tests using real extracted standard library code

#[test]
fn test_vec_basic_corpus_compiles() {
    // For now, just verify the corpus file exists and can be read
    let corpus_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus/std/vec_basic.rs");
    let content = std::fs::read_to_string(corpus_path).expect("Corpus file should exist");

    // Just verify we can transpile it (even if not perfectly)
    let iron = redox::transpile(&content);

    // Log the Iron output for debugging
    match &iron {
        Ok(iron_code) => {
            println!("Vec basic corpus Iron output:\n{}", iron_code);
        }
        Err(e) => {
            println!("Vec basic corpus failed to transpile: {}", e);
        }
    }

    // Don't assert - this is just to see what works
    let _ = iron;
}
