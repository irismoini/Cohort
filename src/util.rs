#[repr(C, align(128))]
pub struct Aligned<T>(pub T);