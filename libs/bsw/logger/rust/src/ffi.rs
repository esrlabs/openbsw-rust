#![allow(static_mut_refs)]
use core::ffi::{CStr, c_char};

unsafe extern "C" {
    pub(crate) fn bsw_cpp_logger_log(
        component_index: u8,
        level: Level,
        str: *const core::ffi::c_char,
    );
}

#[unsafe(no_mangle)]
static mut LOGGER: crate::BswLogger = crate::BswLogger::create();

/// provide information about the loggers to rust
#[unsafe(no_mangle)]
pub extern "C" fn add_component_info_array(
    number_of_components: usize,
    component_info_array: *const PlainLoggerMappingInfo,
) {
    let components = if number_of_components == 0 {
        &[]
    } else {
        unsafe { core::slice::from_raw_parts(component_info_array, number_of_components) }
    };
    unsafe {
        LOGGER.add_components(components);
    };
}

/// Map all rust logs from one crate to a given logger component
#[unsafe(no_mangle)]
pub extern "C" fn map_crate_to_component(
    crate_: *const c_char,
    logger_component: *const c_char,
) -> u8 {
    match unsafe {
        LOGGER.map_crate_to_component(
            CStr::from_ptr(crate_),
            CStr::from_ptr(logger_component)
                .to_str()
                .expect("only valid utf-8 allowed"),
        )
    } {
        Ok(()) => 0,
        Err(crate::bsw_logger::Error::ComponentNotFound) => 1,
        Err(crate::bsw_logger::Error::NoSpaceLeft) => 2,
    }
}

pub(crate) type ComponentIndex = u8;

/// The rust logger must be installed after configuration
#[unsafe(no_mangle)]
pub extern "C" fn install_rust_logging() {
    unsafe { LOGGER.install_logger() };
}

/// cbindgen:no-export
#[repr(C)]
pub(crate) struct PlainLoggerMappingInfo {
    logger_component: *const u8,
    component_info: PlainInfo,
    initial_level: u8,
}

/// cbindgen:no-export
#[repr(C)]
struct PlainInfo {
    name_info: PlainAttributedString,
}

/// cbindgen:no-export
#[repr(C)]
struct PlainAttributedString {
    string: *const c_char,
    attributes: StringAttributes,
}

/// cbindgen:no-export
#[repr(C)]
struct StringAttributes {
    attributes: PlainStringAttributes,
}

/// cbindgen:no-export
#[repr(C)]
struct PlainStringAttributes {
    foreground_color: Color,
    format: u8,
    backgrount_color: Color,
}

/// cbindgen:no-export
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
#[repr(u8)]
enum Color {
    DefaultColor = 0,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    LightGray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,

    NumberOfColors,
}

impl PlainLoggerMappingInfo {
    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.component_info.name_info.string) }
    }
}

#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
/// cbindgen:no-export
#[repr(C)]
#[derive(Debug)]
pub(crate) enum Level {
    LevelDebug = 0,
    LevelInfo = 1,
    LevelWarn = 2,
    LevelError = 3,
    LevelCritical = 4,
    LevelNone = 5,
}
