#![no_std]
use core::marker::PhantomData;

/// The [Embassy](embassy_executor::raw::Executor) based executor for OpenBSW.
///
/// This executor is designed to be run in an Async Runnable. This allows the concurrent execution
/// of C++ and Rust in the same RTOS thread. Thus creating no additional boundaries for the gradual
/// adoption of Rust into all areas of OpenBSW.
/// Another benefit of this approach is the platform independence of OpenBSW is maintained.
///
/// # Caveats
/// Embassy is designed to execute all queued tasks. This means the Runnable hosting the [BswExecutor]
/// might introduce higher jitter in other Runnables running on the same RTOS thread.
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
    /// Create a new [BswExecutor] for a given RTOS thread/task id.
    pub fn new(task_id: u8) -> Self {
        Self {
            inner: embassy_executor::raw::Executor::new(task_id as *mut ()),
            not_send: PhantomData,
        }
    }

    /// Poll all queued tasks in this executor.
    ///
    /// This loops over all tasks that are queued to be polled (i.e. they're
    /// freshly spawned or they've been woken). Other tasks are not polled.
    /// Each task is only polled once.
    ///
    /// You must call `poll` after receiving a call to the pender. It is OK
    /// to call `poll` even when not requested by the pender, but it wastes
    /// energy.
    ///
    /// # Safety
    ///
    /// You must call [Self::new()] before calling this method.
    ///
    /// You must NOT call `poll` reentrantly on the same executor.
    ///
    /// In particular, note that `poll` may call the pender synchronously. Therefore, you
    /// must NOT directly call `poll()` from the pender callback. Instead, the callback has to
    /// somehow schedule for `poll()` to be called later, at a time you know for sure there's
    /// no `poll()` already running.
    pub unsafe fn poll(&'static self) {
        unsafe { self.inner.poll() };
    }

    /// Run the executor.
    ///
    /// The `init` closure is called with a [`embassy_executor::Spawner`] that spawns tasks on
    /// this executor. Use it to spawn the initial task(s). After `init` returns,
    /// the executor starts running the tasks.
    ///
    /// To spawn more tasks later, you may keep copies of the [`embassy_executor::Spawner`] (it is `Copy`),
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
