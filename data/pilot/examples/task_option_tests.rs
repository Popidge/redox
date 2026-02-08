#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    fn returns_some() {
        assert_eq!(maybe_answer(), Some(42));
    }
}

fn maybe_answer() -> Option<i32> {
    Some(42)
}
