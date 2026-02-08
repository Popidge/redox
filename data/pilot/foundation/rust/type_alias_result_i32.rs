type MyResult<T> = Result<T, ()>;

pub fn type_alias_result_i32() -> MyResult<i32> {
    Ok(42)
}
