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
    sender: CohortFifo<T>,
    receiver: CohortFifo<T>,
    acc: Aligned<AtomicU64>,
    // Prevents compiler from implementing unpin trait
    _pin: PhantomPinned,
}

impl<T: Copy> Cohort<T> {
    pub fn register(capacity: usize) -> Pin<Box<Self>> {
        let sender = CohortFifo::new(capacity);
        let receiver = CohortFifo::new(capacity);
        let acc = Aligned(AtomicU64::new(0));

        let cohort = Box::pin(Cohort {
            sender,
            receiver,
            acc,
            _pin: PhantomPinned,
        });

        unsafe {
            libc::syscall(
                258,
                &cohort.sender,
                &cohort.receiver,
                &(cohort.acc.0),
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
