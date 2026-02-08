fn compare_greater(x: i32, y: i32) -> bool {
    x > y
}


fn generic_bound<T: Default>() -> T {
    T.default()
}


fn greater_than(a: i32, b: i32) -> bool {
    a > b
}


fn tail_expr() -> Option<i32> {
    Some(42)
}


fn return_stmt() -> Option<i32> {
    return Some(42);
}


fn complex_types(x: &mut [i32]) -> Option<&i32> {
    x.first()
}


fn simple_loop(n: i32) -> i32 {
    let mut sum = 0;
    let mut i = 0;
    while i < n {
        sum = sum + i;
        i = i + 1;
    }
    sum
}


fn for_loop(items: &[i32]) -> i32 {
    let mut sum = 0;
    for item in items {
        sum = sum + item;
    }
    sum
}
