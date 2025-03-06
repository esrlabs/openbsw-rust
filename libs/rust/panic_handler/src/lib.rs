#![no_std]
//! Installs `softwareSystemReset` as the panic handler on no_std targets.
//! On std targets the std panic handler is used.
//!
//! # Usage
//! To install this panic handler just declare it in you crate:
//! ```
//! extern crate openbsw_panic_handler;
//! ```

#[cfg(not(target_os = "none"))]
extern crate std;

#[cfg(target_os = "none")]
use core::panic::PanicInfo;

#[cfg(target_os = "none")]
unsafe extern "C" {
    safe fn softwareSystemReset() -> !;
}

#[cfg(target_os = "none")]
#[panic_handler]
/// Calls softwareSystemReset from bspMCU
fn panic(_info: &PanicInfo) -> ! {
    softwareSystemReset()
}
