pub fn result_unwrap_or_zero(input: Result<i32, String>) -> i32 {
    input.unwrap_or(0)
}
