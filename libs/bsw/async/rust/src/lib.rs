#![no_std]
use core::marker::PhantomData;

pub struct BswExecutor {
    inner: embassy_executor::raw::Executor,
    not_send: PhantomData<*mut ()>,
}

#[cfg(not(test))]
unsafe extern "C" {
    safe fn schedule_rust_runtime(context: u8);
}

#[cfg(not(test))]
#[unsafe(export_name = "__pender")]
fn __pender(context: *mut ()) {
    schedule_rust_runtime(context as u8);
}

impl BswExecutor {
    pub fn new() -> Self {
        const TASK_DEMO: u8 = 4;
        Self {
            inner: embassy_executor::raw::Executor::new(TASK_DEMO as *mut ()),
            not_send: PhantomData,
        }
    }

    pub unsafe fn poll(&'static self) {
        unsafe { self.inner.poll() };
    }

    /// Run the executor.
    ///
    /// The `init` closure is called with a [`Spawner`] that spawns tasks on
    /// this executor. Use it to spawn the initial task(s). After `init` returns,
    /// the executor starts running the tasks.
    ///
    /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is `Copy`),
    /// for example by passing it as an argument to the initial tasks.
    ///
    /// This function requires `&'static mut self`. This means you have to store the
    /// Executor instance in a place where it'll live forever and grants you mutable
    /// access. There's a few ways to do this:
    ///
    /// - a [StaticCell](https://docs.rs/static_cell/latest/static_cell/) (safe)
    /// - a `static mut` (unsafe)
    /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
    pub fn init(&'static mut self, init: impl FnOnce(embassy_executor::Spawner)) {
        init(self.inner.spawner());

        unsafe {
            self.inner.poll();
        };
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
