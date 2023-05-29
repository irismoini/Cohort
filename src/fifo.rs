use crate::util::Aligned;
use core::ptr::NonNull;

#[repr(packed)]
pub struct Meta<T> {
    addr: *mut T,
    size: u32,
    len: u32,
}

#[repr(C)]
pub struct CohortFifo<T: Copy> {
    head: Aligned<NonNull<T>>,
    meta: Aligned<NonNull<Meta<T>>>,
    tail: Aligned<NonNull<T>>,
}

impl<T: Copy> CohortFifo<T> {
    // Creates new fifo.
    pub fn new(capacity: usize) -> Self {
        todo!();
    }

    /// Pushes an element to the fifo.
    pub fn push(&self, elem: T) {
        todo!();
    }

    /// Pops an element from the fifo.
    pub fn pop(&self) -> T {
        todo!();
    }
}

impl<T: Copy> Drop for CohortFifo<T> {
    fn drop(&mut self) {
        todo!();
    }
}
