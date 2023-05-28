#![feature(atomic_from_mut)]

pub(crate) mod fail;
pub(crate) mod raw_array;
pub(crate) mod ring;
pub(crate) mod util;

use std::marker::PhantomPinned;
use std::pin::Pin;
use std::sync::atomic::AtomicU64;

use crate::ring::RingBuffer;
use crate::util::Aligned;

const BACKOFF_COUNTER_VAL: u64 = 240;

struct Cohort<T> {
    sender: RingBuffer<T>,
    receiver: RingBuffer<T>,
    acc: Aligned<AtomicU64>,
    // Prevents compiler from implementing unpin trait
    _pin: PhantomPinned
}

impl<T: Copy> Cohort<T> {
    pub fn register(capacity: usize) -> Pin<Box<Self>> {
        let sender = RingBuffer::new(capacity).unwrap();
        let receiver = RingBuffer::new(capacity).unwrap();
        let acc = Aligned(AtomicU64::new(0));

        let cohort = Box::pin(Cohort {
            sender,
            receiver,
            acc,
            _pin: PhantomPinned
        });

        unsafe {
            libc::syscall(
                258,
                cohort.sender.get_front_ptr(),
                cohort.receiver.get_back_ptr(),
                &(cohort.acc.0),
                BACKOFF_COUNTER_VAL,
            );
        }

        cohort
    }

    /// Sends an element to the accelerator.
    pub fn push(&self, elem: T) {
        todo!();
    }

    /// Reads an element from the accelerator.
    pub fn pop(&self) -> T {
        todo!();
    }
}

impl<T> Drop for Cohort<T> {
    fn drop(&mut self) {
        // Unregister thorugh specific syscall.
    }
}
