#![forbid(unsafe_code)]

//! `next-loggers/v1` diagnostics for the `ores-otel` CLI.
//!
//! stdout belongs to command output, so the SDK console printer is disabled and every record is
//! written as one JSON line to stderr. Records carry outcome metadata only (command name, error
//! class, exit code): never argv, flag values, API base URLs, or error messages, because those can
//! echo operator input.

use std::io::Write;
use std::sync::Arc;

use next_loggers::{LogLevel, LogRecord, Logger, LoggerError, Options, Transport};

/// `appName` stamped on every record.
pub const APP_NAME: &str = "ores-otel-cli";

/// Writes each record as a single JSON line to stderr.
#[derive(Clone, Copy, Debug, Default)]
pub struct StderrJsonTransport;

impl Transport for StderrJsonTransport {
    fn write(&self, record: &LogRecord) -> Result<(), LoggerError> {
        let line = record.to_json()?;
        writeln!(std::io::stderr().lock(), "{line}")
            .map_err(|error| LoggerError(error.to_string()))
    }
}

/// Logger options for the CLI: no console printer, records at or above `max_level` only.
pub fn options(max_level: LogLevel) -> Options {
    Options {
        app_name: APP_NAME.into(),
        max_level,
        console: false,
        ..Options::default()
    }
}

/// The process logger. Success-path records are emitted at debug, so the default `Info` level
/// leaves normal command output unchanged; failures are emitted at error.
pub fn logger() -> Logger {
    Logger::new(options(LogLevel::Info).with_transport(Arc::new(StderrJsonTransport)))
}
