use crate::util::Aligned;
use core::ptr::NonNull;
use std::{
    alloc::{alloc, dealloc, Layout},
    cell::UnsafeCell,
    mem, ptr,
};

#[repr(packed)]
pub struct Meta<T> {
    buffer: NonNull<T>,
    _elem_size: u32, // TODO: Determine what this actually is in cohort
    capacity: u32,   // TODO: Determine what this actually is in cohort
}

#[repr(C)]
pub struct CohortFifo<T: Copy> {
    head: Aligned<UnsafeCell<u32>>,
    meta: Aligned<Meta<T>>,
    tail: Aligned<UnsafeCell<u32>>,
}

impl<T: Copy> CohortFifo<T> {
    // Creates new fifo.
    pub fn new(capacity: usize) -> Self {
        let buffer = unsafe {
            let buffer_size = capacity + 1;
            let layout = Layout::array::<T>(buffer_size).unwrap();
            let aligned = layout.align_to(128).unwrap();
            NonNull::new(alloc(aligned)).unwrap()
        };

        CohortFifo {
            head: Aligned(UnsafeCell::new(0)),
            meta: Aligned(Meta {
                buffer: buffer.cast(),
                _elem_size: mem::size_of::<T>() as u32,
                capacity: capacity as u32,
            }),
            tail: Aligned(UnsafeCell::new(0)),
        }
    }

    pub fn try_push(&self, elem: T) -> Result<(), T> {
        if self.is_full() {
            return Err(elem);
        }

        let tail = self.tail();
        unsafe {
            (*self.buffer().as_ptr())[tail] = elem;
        }
        self.set_tail((tail + 1) % self.capacity());

        Ok(())
    }

    /// Pushes an element to the fifo.
    pub fn push(&self, elem: T) {
        while self.try_push(elem).is_err() {}
    }

    pub fn try_pop(&self) -> Result<T, ()> {
        if self.is_empty() {
            return Err(());
        }

        let head = self.head();
        let elem = unsafe { (*self.buffer().as_ptr())[head] };
        self.set_head((head + 1) % self.capacity());

        Ok(elem)
    }

    /// Pops an element from the fifo.
    pub fn pop(&self) -> T {
        loop {
            if let Ok(data) = self.try_pop() {
                break data;
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.meta.0.capacity as usize
    }

    /// True size of the underlying buffer.
    // Should always be one more than the given capacity.
    // The extra allocated slot in the buffer is used to determine whether the buffer is full (it does not hold an additional element).
    fn buffer_size(&self) -> usize {
        (self.meta.0.capacity + 1) as usize
    }

    pub fn is_full(&self) -> bool {
        (self.head() % self.buffer_size()) == ((self.tail() + 1) % self.buffer_size())
    }

    pub fn is_empty(&self) -> bool {
        self.head() == self.tail()
    }

    fn head(&self) -> usize {
        unsafe { ptr::read_volatile(self.head.0.get()) as usize }
    }

    fn tail(&self) -> usize {
        unsafe { ptr::read_volatile(self.tail.0.get()) as usize }
    }

    fn set_head(&self, head: usize) {
        unsafe {
            ptr::write_volatile(self.head.0.get(), head as u32);
        }
    }

    fn set_tail(&self, tail: usize) {
        unsafe {
            ptr::write_volatile(self.tail.0.get(), tail as u32);
        }
    }

    fn buffer(&self) -> NonNull<[T]> {
        NonNull::slice_from_raw_parts(self.meta.0.buffer, self.capacity())
    }
}

unsafe impl<T: Copy> Send for CohortFifo<T> {}

impl<T: Copy> Drop for CohortFifo<T> {
    fn drop(&mut self) {
        let layout = Layout::array::<T>(self.buffer_size()).unwrap();
        let aligned = layout.align_to(128).unwrap();
        unsafe { dealloc(self.meta.0.buffer.cast().as_ptr(), aligned) };
    }
}
