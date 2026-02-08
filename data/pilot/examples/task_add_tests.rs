#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    fn adds_values() {
        assert_eq!(add(2, 3), 5);
    }
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
