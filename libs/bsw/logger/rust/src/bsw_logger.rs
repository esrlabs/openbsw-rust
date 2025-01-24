use crate::ffi;

pub struct BswLogger {
    pub(crate) components: &'static [crate::ffi::PlainLoggerMappingInfo],
    pub(crate) crate_to_component_index:
        [Option<(&'static core::ffi::CStr, crate::ffi::ComponentIndex)>; 128],
}

// TODO: Check safety
unsafe impl Send for BswLogger {}
unsafe impl Sync for BswLogger {}

impl BswLogger {
    /// Create a new [BswLogger] without any forwarding configured
    pub const fn new() -> Self {
        Self {
            components: &[],
            crate_to_component_index: [None; 128],
        }
    }

    /// Redirect all log messages produced in a crate to an OpenBSW logger component
    pub fn map_crate_to_component(
        &mut self,
        crate_: &'static core::ffi::CStr,
        logger_component: &str,
    ) -> Result<(), Error> {
        assert!(crate_.to_str().is_ok());
        if let Some(component_index) = self.find_logger_component_index(logger_component) {
            if let Some(next_free) = self
                .crate_to_component_index
                .iter_mut()
                .find(|m| m.is_none())
            {
                *next_free = Some((crate_, component_index));
                Ok(())
            } else {
                Err(Error::NoSpaceLeft)
            }
        } else {
            Err(Error::ComponentNotFound)
        }
    }

    /// Install the [BswLogger] as the system wide logger
    pub fn install_logger(&'static self) {
        log::set_logger(self).expect("failed to set logger");
        log::set_max_level(log::LevelFilter::Trace);
    }

    /// Add the components defined on the CPP side
    pub(crate) fn add_components(
        &mut self,
        components: &'static [crate::ffi::PlainLoggerMappingInfo],
    ) {
        self.components = components;
    }

    fn component_for_crate(&self, crate_name: &str) -> Option<crate::ffi::ComponentIndex> {
        for entry in self.crate_to_component_index {
            match entry {
                None => return None,
                Some((name, index)) if name.to_str().expect("needs to be utf-8") == crate_name => {
                    return Some(index);
                }
                _ => {}
            }
        }
        None
    }

    /// Find the component index for a given component name
    fn find_logger_component_index(
        &self,
        component_name: &str,
    ) -> Option<crate::ffi::ComponentIndex> {
        self.components
            .iter()
            .enumerate()
            .filter_map(|(index, ci)| ci.name().to_str().ok().map(|name| (index, name)))
            .find(|(_index, name)| *name == component_name)
            .map(|(index, _name)| index as crate::ffi::ComponentIndex)
    }
}

impl log::Log for BswLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        // checked in cpp
        true
    }

    fn log(&self, record: &log::Record) {
        use core::fmt::Write;
        let crate_ = record.target().split("::").next().unwrap();
        if let Some(component_index) = self
            .component_for_crate(crate_)
            .or_else(|| self.find_logger_component_index(crate_))
        {
            let mut buf = crate::log_buf::LogBuf::<400>::default();
            // Message might be truncated
            let _ = write!(buf, "{}", record.args());
            let level = {
                use ffi::Level::*;
                use log::Level::*;
                match record.level() {
                    Error => LevelError,
                    Warn => LevelWarn,
                    Info => LevelInfo,
                    Debug => LevelDebug,
                    Trace => LevelNone,
                }
            };
            unsafe { crate::ffi::bsw_cpp_logger_log(component_index, level, buf.as_raw_pointer()) };
        } else {
            // No logger defined for target. Falling back to directly printing it to stdout
            write!(
                console_out::Console,
                "{}: {}: {}",
                record.target(),
                record.level(),
                record.args()
            )
            .and_then(|()| write!(console_out::Console, "\r\n"))
            .expect("printing to stdout should always be available");
        }
    }

    fn flush(&self) {}
}
#[derive(Debug)]
pub enum Error {
    ComponentNotFound,
    NoSpaceLeft,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::ComponentNotFound => "No component found for the crate",
                Error::NoSpaceLeft => "No space left in crate to component mapping",
            }
        )
    }
}

impl core::error::Error for Error {}
