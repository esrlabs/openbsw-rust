#![no_std]
//! The Embassy Time Driver implementation for OpenBSW
//!
//! Currently there is a fixed number of queues. There needs to be one queue per executor.
//!
//! TODO:
//! One queue per executor shouldn't require any locking, because the each queue would
//! only be accessed form within one FreeRtos task.
use core::cell::RefCell;
use core::time::Duration;

use critical_section::Mutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

// FIXME: Should this be based on a crate feature?
const NUMBER_OF_TASKS: usize = 8;

struct BswTimerDriver {
    queues: [Mutex<RefCell<Queue>>; NUMBER_OF_TASKS],
}

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
        critical_section::with(|cs| {
            let mut queue = self.queues[getCurrentTaskContext() as usize]
                .borrow(cs)
                .borrow_mut();
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
        critical_section::with(|cs| {
            let mut queue = self.queues[getCurrentTaskContext() as usize]
                .borrow(cs)
                .borrow_mut();
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

embassy_time_driver::time_driver_impl!(static DRIVER: BswTimerDriver = BswTimerDriver{ queues: [const{Mutex::new(RefCell::new(Queue::new()))}; NUMBER_OF_TASKS] });
