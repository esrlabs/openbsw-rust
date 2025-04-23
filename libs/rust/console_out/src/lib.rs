#![no_std]

extern crate openbsw_panic_handler;

/// Stdout console
///
/// implements [`core::fmt::write`]
pub struct Console;

impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            unsafe {
                putByteToStdout(b);
            }
        }
        Ok(())
    }
}

unsafe extern "C" {
    fn putByteToStdout(byte: u8);
}
