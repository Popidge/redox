// Standard library Vec tests - Basic operations
// Extracted from std::vec module

pub fn vec_new() -> Vec<i32> {
    Vec::new()
}

pub fn vec_with_capacity() -> Vec<i32> {
    Vec::with_capacity(10)
}

pub fn vec_push() -> Vec<i32> {
    let mut v = Vec::new();
    v.push(42);
    v
}

pub fn vec_pop() -> Option<i32> {
    let mut v = vec![1, 2, 3];
    v.pop()
}

pub fn vec_get() -> Option<&'static i32> {
    let v = vec![1, 2, 3];
    v.get(0)
}

pub fn vec_len() -> usize {
    let v = vec![1, 2, 3];
    v.len()
}
