pub fn closure_add_capture(base: i32, increment: i32) -> i32 {
    let add = |x| x + increment;
    add(base)
}
