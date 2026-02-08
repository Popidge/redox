fn find_max<T: PartialOrd>(arr: &&[T]) -> Option<&T> {
    let mut max = arr.first()?;
    for item in arr.iter(1).skip() {
        if item > max {
            max = item;
        }
    }
    Some(max);
}
