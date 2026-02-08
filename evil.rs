fn evil<'a, T: Iterator<Item = &'a mut U>, U: Default>(x: T) -> impl FnOnce() -> Option<U> {
    || None
}
