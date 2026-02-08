// Simplified torture test for round-trip validation

// Test 1: Greater than comparison
fn compare_greater(x: i32, y: i32) -> bool {
    x > y
}

// Test 2: Generic bounds
fn generic_bound<T: Default>() -> T {
    T::default()
}

// Test 3: User identifier with keywords
fn greater_than(a: i32, b: i32) -> bool {
    a > b
}

// Test 4: Tail expression vs return statement
fn tail_expr() -> Option<i32> {
    Some(42)
}

fn return_stmt() -> Option<i32> {
    return Some(42);
}

// Test 5: Complex types
fn complex_types(x: &mut [i32]) -> Option<&i32> {
    x.first()
}

// Test 6: Simple loop
fn simple_loop(n: i32) -> i32 {
    let mut sum = 0;
    let mut i = 0;
    while i < n {
        sum = sum + i;
        i = i + 1;
    }
    sum
}

// Test 7: For loop
fn for_loop(items: &[i32]) -> i32 {
    let mut sum = 0;
    for item in items {
        sum = sum + item;
    }
    sum
}
