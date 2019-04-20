//! A log::Log implementation that sends data back and forth to WASM. Useful for debugging and
//! reporting the status of things.
//!
//! Largely copied from: https://github.com/iamcodemaker/console_log
//! Adapted to work with our WASM imports/exports instead of web_sys.

use std::ffi::CString;
use std::os::raw::c_char;

use log::{Log, Level, Record, Metadata, SetLoggerError};

static LOGGER: ConsoleLogger = ConsoleLogger;

// Functions provided by JavaScript, to be called by the WebAssembly generated from Rust
extern {
    fn console_error(message: *const c_char);
    fn console_warn(message: *const c_char);
    fn console_info(message: *const c_char);
    fn console_log(message: *const c_char);
    fn console_debug(message: *const c_char);
}

struct ConsoleLogger;

impl Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    /// Print a `log::Record` to the browser's console at the appropriate level.
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // pick the console.log() variant for the appropriate logging level
        let console_log = match record.level() {
            Level::Error => console_error,
            Level::Warn => console_warn,
            Level::Info => console_info,
            Level::Debug => console_log,
            Level::Trace => console_debug,
        };

        let formatted = format!("{}", record.args());
        let raw_str = CString::new(formatted).expect("CString::new failed");
        unsafe { console_log(raw_str.into_raw()) };
    }

    fn flush(&self) {}
}

/// Initializes the global logger setting `max_log_level` to the given value.
///
/// ## Example
///
/// ```ignore
/// use log::Level;
/// fn main() {
///     console_log::init_with_level(Level::Debug).expect("error initializing logger");
/// }
/// ```
pub fn init_with_level(level: Level) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)?;
    log::set_max_level(level.to_level_filter());
    Ok(())
}

/// Initializes the global logger with `max_log_level` set to `Level::Info` (a sensible default).
///
/// ## Example
///
/// ```ignore
/// fn main() {
///     console_log::init().expect("error initializing logger");
/// }
/// ```
pub fn init() -> Result<(), SetLoggerError> {
    init_with_level(Level::Info)
}
