type Alias010<T> = Result<T, ()>;

pub fn alias_result_010() -> Alias010<i32> {
    Ok(210)
}
