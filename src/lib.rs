//! Logger implementation for low level kernel log (using `/dev/kmsg`)
//!
//! Usually intended for low level implementations, like [systemd generators][1],
//! which have to use `/dev/kmsg`:
//!
//! > Since syslog is not available (see above) write log messages to /dev/kmsg instead.
//!
//! [1]: http://www.freedesktop.org/wiki/Software/systemd/Generators/
//!
//! # Examples
//!
//! ```toml
//! [dependencies]
//! log = "*"
//! kernlog = "*"
//! ```
//! 
//! ```rust
//! #[macro_use]
//! extern crate log;
//! extern crate kernlog;
//! 
//! fn main() {
//!     kernlog::init().unwrap();
//!     warn!("something strange happened");
//! }
//! ```
//! Note you have to have permissions to write to `/dev/kmsg`,
//! which normal users (not root) usually don't.
//! 
//! If compiled with nightly it can use libc feature to get process id
//! and report it into log. This feature is unavailable for stable release
//! for now. To enable nightly features, compile with `--features nightly`:
//!
//! ```toml
//! [dependencies.kernlog]
//! version = "*"
//! features = ["nightly"]
//! ```

#![deny(missing_docs)]
#![cfg_attr(feature="nightly", feature(libc))]

#[macro_use]
extern crate log;
#[cfg(feature="nightly")]
extern crate libc;

use std::fs::{OpenOptions, File};
use std::io::Write;
use std::sync::Mutex;

use log::{Log, LogMetadata, LogRecord, LogLevel, MaxLogLevelFilter, LogLevelFilter, SetLoggerError};

/// Kernel logger implementation
pub struct KernelLog {
    kmsg: Mutex<File>
}

impl KernelLog {
    /// Create new kernel logger
    pub fn new() -> KernelLog {
        KernelLog {
            kmsg: Mutex::new(OpenOptions::new().write(true).open("/dev/kmsg").unwrap())
        }
    }

    /// Setup new kernel logger for log framework
    pub fn init(filter: MaxLogLevelFilter) -> Box<Log> {
        filter.set(LogLevelFilter::Trace);
        Box::new(KernelLog::new())
    }
}

impl Log for KernelLog {
    fn enabled(&self, _meta: &LogMetadata) -> bool {
        true
    }

    #[cfg(feature="nightly")]
    fn log(&self, record: &LogRecord) {
        let level: u8 = match record.level() {
            LogLevel::Error => 3,
            LogLevel::Warn => 4,
            LogLevel::Info => 5,
            LogLevel::Debug => 6,
            LogLevel::Trace => 7,
        };

        let mut buf = Vec::new();
        writeln!(buf, "<{}>{}[{}]: {}", level, record.target(),
                 unsafe { ::libc::funcs::posix88::unistd::getpid() },
                 record.args()).unwrap();

        if let Ok(mut kmsg) = self.kmsg.lock() {
            let _ = kmsg.write(&buf);
            let _ = kmsg.flush();
        }
    }

    #[cfg(not(feature="nightly"))]
    fn log(&self, record: &LogRecord) {
        let level: u8 = match record.level() {
            LogLevel::Error => 3,
            LogLevel::Warn => 4,
            LogLevel::Info => 5,
            LogLevel::Debug => 6,
            LogLevel::Trace => 7,
        };

        let mut buf = Vec::new();
        writeln!(buf, "<{}>{}: {}", level, record.target(), record.args()).unwrap();

        if let Ok(mut kmsg) = self.kmsg.lock() {
            let _ = kmsg.write(&buf);
            let _ = kmsg.flush();
        }
    }
}

/// Setup kernel logger as a default logger
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(KernelLog::init)
}

#[cfg(test)]
mod tests {
    use super::{KernelLog, init};

    #[test]
    fn log_to_kernel() {
        init().unwrap();
        debug!("hello, world!");
    }
}
