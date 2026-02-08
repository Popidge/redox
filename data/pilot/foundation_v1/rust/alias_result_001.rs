type Alias001<T> = Result<T, ()>;

pub fn alias_result_001() -> Alias001<i32> {
    Ok(201)
}
