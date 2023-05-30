#![feature(atomic_from_mut)]

mod fifo;
pub(crate) mod util;

use core::marker::PhantomPinned;
use core::pin::Pin;
use core::sync::atomic::AtomicU64;

use fifo::CohortFifo;

use crate::util::Aligned;

const BACKOFF_COUNTER_VAL: u64 = 240;

pub struct Cohort<T: Copy> {
    id: u8,
    sender: CohortFifo<T>,
    receiver: CohortFifo<T>,
    custom_data: Aligned<AtomicU64>, //TODO: Determine type
    // Prevents compiler from implementing unpin trait
    _pin: PhantomPinned,
}

impl<T: Copy> Cohort<T> {
    pub fn register(id: u8, capacity: usize) -> Pin<Box<Self>> {
        let sender = CohortFifo::new(capacity);
        let receiver = CohortFifo::new(capacity);
        let custom_data = Aligned(AtomicU64::new(0));

        let cohort = Box::pin(Cohort {
            id,
            sender,
            receiver,
            custom_data,
            _pin: PhantomPinned,
        });

        unsafe {
            libc::syscall(
                258,
                &cohort.sender,
                &cohort.receiver,
                &(cohort.custom_data.0),
                BACKOFF_COUNTER_VAL,
            );
        }

        cohort
    }

    /// Sends an element to the accelerator.
    pub fn push(&self, elem: T) {
        self.sender.push(elem);
    }

    /// Reads an element from the accelerator.
    pub fn pop(&self) -> T {
        self.receiver.pop()
    }
}

impl<T: Copy> Drop for Cohort<T> {
    fn drop(&mut self) {
        unsafe {
            //TODO: check status from syscall
            libc::syscall(257);
        }
    }
}
