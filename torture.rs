// Torture test for round-trip fidelity

// Test 1: Greater than in different contexts
fn compare_greater(x: i32, y: i32) -> bool {
    x > y
}

fn generic_bound<T: Ord>() -> T
where
    T: Default,
{
    T::default()
}

// Test 2: User-defined identifier with keywords
fn greater_than(a: i32, b: i32) -> bool {
    a > b
}

fn mutable_reference() -> i32 {
    42
}

// Test 3: Nested generics with comparisons
fn nested<T: Iterator<Item = U>, U: PartialOrd>(iter: T) -> Option<U> {
    let mut max = None;
    for item in iter {
        if let Some(ref m) = max {
            if item > *m {
                max = Some(item);
            }
        } else {
            max = Some(item);
        }
    }
    max
}

// Test 4: Closure preservation
fn make_closure() -> impl Fn(i32) -> i32 {
    |x| x * 2
}

// Test 5: Tail expressions
fn tail_expr() -> Option<i32> {
    Some(42)
}

fn return_stmt() -> Option<i32> {
    return Some(42);
}

// Test 6: Complex types
fn complex_types(x: &mut [i32]) -> Option<&i32> {
    x.first()
}
