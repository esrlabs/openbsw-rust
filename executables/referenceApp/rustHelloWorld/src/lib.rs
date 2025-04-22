#![no_std]
#![allow(static_mut_refs)]

#[cfg(target_os = "none")]
extern crate cortex_m;

extern crate openbsw_panic_handler;

use core::mem::MaybeUninit;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Ticker, Timer, block_for};

use log::{error, info, warn};
use openbsw_can::{CanFrame, CanFrameTrait};
use openbsw_runtime::BswExecutor;
use openbsw_timer as _;
use reference_app_async_core_configuration::{TASK, TASK_TASK_BACKGROUND, TASK_TASK_DEMO};

static mut DEMO_EXECUTOR: MaybeUninit<openbsw_runtime::BswExecutor> = MaybeUninit::uninit();
static mut BACKGROUND_EXECUTOR: MaybeUninit<openbsw_runtime::BswExecutor> = MaybeUninit::uninit();

/// Initializes the async rust runtime for the demo task
#[unsafe(no_mangle)]
pub extern "C" fn init_rust_runtime(context: u8) {
    match context as TASK {
        reference_app_async_core_configuration::TASK_TASK_DEMO => {
            unsafe {
                DEMO_EXECUTOR
                    .write(BswExecutor::new(TASK_TASK_DEMO as u8))
                    .init(|spawner| {
                        spawner.must_spawn(loopy());
                        spawner.must_spawn(loopy2());
                        spawner.must_spawn(can_task());
                    });
            };
        }
        reference_app_async_core_configuration::TASK_TASK_BACKGROUND => {
            unsafe {
                BACKGROUND_EXECUTOR
                    .write(BswExecutor::new(TASK_TASK_BACKGROUND as u8))
                    .init(|spawner| {
                        spawner.must_spawn(background());
                    });
            };
        }
        _ => unreachable!(),
    }
}

/// This is the entry point which needs to be called to drive the runtime
///
/// # Safety
///
/// The runtime must be initialized before calling this function the first time.
///
/// The given context must correspond to a valid freertos task with a rust tuntime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn runtime_poll(context: u8) {
    match context as TASK {
        reference_app_async_core_configuration::TASK_TASK_DEMO => {
            let executor = unsafe { DEMO_EXECUTOR.assume_init_mut() };
            unsafe { executor.poll() };
        }
        reference_app_async_core_configuration::TASK_TASK_BACKGROUND => {
            let executor = unsafe { BACKGROUND_EXECUTOR.assume_init_mut() };
            unsafe { executor.poll() };
        }
        _ => unreachable!(),
    }
}

static CURRENT_CAN_FRAME: Watch<CriticalSectionRawMutex, CanFrame, 1> = Watch::new();

#[unsafe(no_mangle)]
/// To be called from C++ once a new CAN Frame arrived.
///
/// # Safety
///
/// Must be given a valid non-null pointer to a CanFrame.
pub unsafe extern "C" fn receive_new_can_frame(frame: *const CanFrame) {
    // Safety: frame must be valid and is copied because CanFrame implements Copy
    let frame = unsafe { *frame };
    CURRENT_CAN_FRAME.sender().send(frame);
}

#[embassy_executor::task]
async fn can_task() {
    let mut can_receiver = CURRENT_CAN_FRAME
        .receiver()
        .expect("can_task not able to subscribe to can receiver");
    loop {
        let frame = can_receiver.changed().await;
        info!("🦀 Received CAN Frame {:?}", frame.id())
    }
}

#[embassy_executor::task]
async fn loopy() {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let mut counter = 0;
    for _ in 0..5 {
        ticker.next().await;
        info!("booting loopy");
        counter += 1;
    }
    info!("Loopy boot completed in {counter} steps");
    loop {
        info!("Doing loopy work");
        ticker.next().await;
    }
}
#[embassy_executor::task]
async fn loopy2() {
    loop {
        warn!("Doing different loopy work");
        Timer::after_secs(2).await;
    }
}

#[embassy_executor::task]
async fn background() {
    loop {
        error!("Starting unimportant long running background job");
        block_for(Duration::from_secs(3));
        error!("DONE");
        Timer::after_secs(5).await;
    }
}
