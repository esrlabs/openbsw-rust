#![no_std]
//! The Embassy Time Driver implementation for OpenBSW
//!
//! Currently there is only one queue as the demo only contains one executor.
//! This needs to be reworked to support multiple executors.
//! It should be possible to have one queue per executor. And probably this wouldn't require
//! any locking, because the each queue would only be accessed form within one FreeRtos task.
use core::cell::RefCell;
use core::time::Duration;

use critical_section::Mutex;
use embassy_time_driver::Driver;
use embassy_time_queue_utils::Queue;

struct BswTimerDriver {
    queue: Mutex<RefCell<Queue>>,
}

unsafe extern "C" {
    safe fn getSystemTimeNs() -> u64;
    safe fn schedule_rust_runtime();
    safe fn schedule_rust_runtime_in(delay_us: u32);
}

#[unsafe(no_mangle)]
pub extern "C" fn check_alarm() {
    DRIVER.check_alarm();
}

impl BswTimerDriver {
    /// reschedule the rust executor
    fn set_alarm(&self, next: u64, now: u64) -> bool {
        if next <= now {
            schedule_rust_runtime();
            false
        } else if next != u64::MAX {
            schedule_rust_runtime_in((next - now) as u32);
            true
        } else {
            true
        }
    }

    fn check_alarm(&self) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
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
            let mut queue = self.queue.borrow(cs).borrow_mut();
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

embassy_time_driver::time_driver_impl!(static DRIVER: BswTimerDriver = BswTimerDriver{ queue: Mutex::new(RefCell::new(Queue::new())) });
