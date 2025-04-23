#![no_std]
//! The Embassy Time Driver implementation for OpenBSW
//!
//! Currently there is a fixed number of queues. There needs to be one queue per executor.
//!
//! To ensure propper linking it's required to use this crate explicitly:
//! ```
//!   extern crate openbsw_timer;
//! ```
use core::cell::RefCell;
use core::time::Duration;

use embassy_sync::blocking_mutex::NoopMutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

#[cfg(feature = "max_1_tasks")]
const NUMBER_OF_TASKS: usize = 1;

#[cfg(feature = "max_2_tasks")]
const NUMBER_OF_TASKS: usize = 2;

#[cfg(feature = "max_3_tasks")]
const NUMBER_OF_TASKS: usize = 3;

#[cfg(feature = "max_4_tasks")]
const NUMBER_OF_TASKS: usize = 4;

#[cfg(feature = "max_5_tasks")]
const NUMBER_OF_TASKS: usize = 5;

#[cfg(feature = "max_6_tasks")]
const NUMBER_OF_TASKS: usize = 6;

#[cfg(feature = "max_7_tasks")]
const NUMBER_OF_TASKS: usize = 7;

#[cfg(feature = "max_8_tasks")]
const NUMBER_OF_TASKS: usize = 8;

#[cfg(feature = "max_9_tasks")]
const NUMBER_OF_TASKS: usize = 9;

#[cfg(feature = "max_10_tasks")]
const NUMBER_OF_TASKS: usize = 10;

#[cfg(feature = "max_11_tasks")]
const NUMBER_OF_TASKS: usize = 11;

#[cfg(feature = "max_12_tasks")]
const NUMBER_OF_TASKS: usize = 12;

#[cfg(feature = "max_13_tasks")]
const NUMBER_OF_TASKS: usize = 13;

#[cfg(feature = "max_14_tasks")]
const NUMBER_OF_TASKS: usize = 14;

#[cfg(feature = "max_15_tasks")]
const NUMBER_OF_TASKS: usize = 15;

#[cfg(feature = "max_16_tasks")]
const NUMBER_OF_TASKS: usize = 16;

#[cfg(feature = "max_17_tasks")]
const NUMBER_OF_TASKS: usize = 17;

#[cfg(feature = "max_18_tasks")]
const NUMBER_OF_TASKS: usize = 18;

#[cfg(feature = "max_19_tasks")]
const NUMBER_OF_TASKS: usize = 19;

#[cfg(feature = "max_20_tasks")]
const NUMBER_OF_TASKS: usize = 20;

#[cfg(not(any(
    feature = "max_1_tasks",
    feature = "max_2_tasks",
    feature = "max_3_tasks",
    feature = "max_4_tasks",
    feature = "max_5_tasks",
    feature = "max_6_tasks",
    feature = "max_7_tasks",
    feature = "max_8_tasks",
    feature = "max_9_tasks",
    feature = "max_10_tasks",
    feature = "max_11_tasks",
    feature = "max_12_tasks",
    feature = "max_13_tasks",
    feature = "max_14_tasks",
    feature = "max_15_tasks",
    feature = "max_16_tasks",
    feature = "max_17_tasks",
    feature = "max_18_tasks",
    feature = "max_19_tasks",
    feature = "max_20_tasks",
)))]
compile_error!("One of the max_x_task features must be enabled!");

struct BswTimerDriver {
    queues: [NoopMutex<RefCell<Queue>>; NUMBER_OF_TASKS],
}

// Safety: Each queue is only ever accessed from one thread.
unsafe impl Sync for BswTimerDriver {}

unsafe extern "C" {
    safe fn getSystemTimeNs() -> u64;
    safe fn schedule_rust_runtime(task_id: u8);
    safe fn schedule_rust_runtime_in(task_id: u8, delay_us: u32);
    safe fn getCurrentTaskContext() -> u8;
}

#[unsafe(no_mangle)]
pub extern "C" fn check_alarm() {
    DRIVER.check_alarm();
}

impl BswTimerDriver {
    /// reschedule the rust executor
    fn set_alarm(&self, next: u64, now: u64) -> bool {
        if next <= now {
            schedule_rust_runtime(getCurrentTaskContext());
            false
        } else if next != u64::MAX {
            schedule_rust_runtime_in(getCurrentTaskContext(), (next - now) as u32);
            true
        } else {
            true
        }
    }

    fn check_alarm(&self) {
        self.queues[getCurrentTaskContext() as usize].lock(|queue| {
            let mut queue = queue.borrow_mut();
            let mut next = queue.next_expiration(self.now());
            while !self.set_alarm(next, self.now()) {
                next = queue.next_expiration(self.now());
            }
        })
    }
}

impl embassy_time_driver::Driver for BswTimerDriver {
    fn now(&self) -> u64 {
        Duration::from_nanos(getSystemTimeNs()).as_micros() as u64
    }

    fn schedule_wake(&self, at: u64, waker: &core::task::Waker) {
        self.queues[getCurrentTaskContext() as usize].lock(|queue| {
            let mut queue = queue.borrow_mut();
            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                while !self.set_alarm(next, self.now()) {
                    next = queue.next_expiration(self.now());
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert!(true);
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: BswTimerDriver = BswTimerDriver{ queues: [const{NoopMutex::new(RefCell::new(Queue::new()))}; NUMBER_OF_TASKS] });
