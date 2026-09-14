#![forbid(unsafe_code)]

//! `next-loggers/v1` diagnostics for the `ores-otel` CLI.
//!
//! stdout belongs to command output, so the SDK console printer is disabled and every record is
//! written as one JSON line to stderr. Records carry outcome metadata only (command name, error
//! class, exit code, output mode): never argv, flag values, API base URLs, or error messages,
//! because those can echo operator input.

use std::io::Write;
use std::sync::Arc;

use next_loggers::{JsonObject, LogLevel, LogRecord, Logger, LoggerError, Options, Transport};
use serde_json::{json, Value};

use crate::args::Command;
use crate::error::CliError;

/// `appName` stamped on every record.
pub const APP_NAME: &str = "ores-otel-cli";

/// Writes each record as a single JSON line to stderr and flushes immediately.
#[derive(Clone, Copy, Debug, Default)]
pub struct StderrJsonTransport;

impl Transport for StderrJsonTransport {
    fn write(&self, record: &LogRecord) -> Result<(), LoggerError> {
        let line = record.to_json()?;
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{line}").map_err(|error| LoggerError(error.to_string()))?;
        stderr
            .flush()
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

/// Process logger. Shared `ores-clis-core` policy performs the authoritative
/// filtering; this backend is kept at Debug so permitted debug records are not
/// filtered a second time by the telemetry implementation.
pub fn logger() -> Logger {
    Logger::new(options(LogLevel::Debug).with_transport(Arc::new(StderrJsonTransport)))
}

/// Stable, input-free label for a parsed command.
pub fn command_name(command: &Command) -> &'static str {
    match command {
        Command::Help => "help",
        Command::Health => "health",
        Command::Status => "status",
    }
}

/// Fields describing which command was dispatched.
pub fn command_fields(command: &Command) -> JsonObject {
    fields([("command", json!(command_name(command)))])
}

/// Fields describing a failed invocation: error class and exit code, never the message.
pub fn failure_fields(error: &CliError) -> JsonObject {
    let kind = match error {
        CliError::Usage(_) => "usage",
        CliError::Config(_) => "config",
        CliError::Command(_) => "command",
    };
    fields([
        ("error.kind", json!(kind)),
        ("exit_code", json!(error.exit_code())),
    ])
}

/// Fields describing how a command rendered its output.
pub fn output_fields(json_output: bool) -> JsonObject {
    fields([("output", json!(if json_output { "json" } else { "text" }))])
}

fn fields<const N: usize>(entries: [(&str, Value); N]) -> JsonObject {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use next_loggers::{LogLevel, Logger, MemoryTransport};
    use serde_json::json;

    use super::*;

    #[test]
    fn failure_fields_carry_class_and_exit_code_but_never_the_message() {
        let fields = failure_fields(&CliError::Usage("unknown flag --token=do-not-log".into()));
        assert_eq!(fields.get("error.kind"), Some(&json!("usage")));
        assert_eq!(fields.get("exit_code"), Some(&json!(2)));
        assert!(!Value::Object(fields).to_string().contains("do-not-log"));
    }

    #[test]
    fn command_fields_use_fixed_labels() {
        assert_eq!(
            command_fields(&Command::Status).get("command"),
            Some(&json!("status"))
        );
        assert_eq!(output_fields(true).get("output"), Some(&json!("json")));
        assert_eq!(output_fields(false).get("output"), Some(&json!("text")));
    }

    #[test]
    fn info_backend_filters_debug_but_runtime_backend_accepts_it() {
        const ROUTINE_ID: &str = "ores-routine-8HNe_nBVinzuvJT9XjRsh";
        let filtered = Arc::new(MemoryTransport::default());
        let info = Logger::new(options(LogLevel::Info).with_transport(filtered.clone()));
        let sent = info
            .debug(vec![json!("filtered")])
            .add_trace("ores-trace-YoMKex_K4A9rydggenVTs", false)
            .add_routine_id(ROUTINE_ID)
            .send();
        assert!(matches!(sent, Ok(None)));
        assert!(filtered.records().is_empty());

        let accepted = Arc::new(MemoryTransport::default());
        let debug = Logger::new(options(LogLevel::Debug).with_transport(accepted.clone()));
        let sent = debug
            .debug(vec![json!("accepted")])
            .add_trace("ores-trace-PxuTkVW-mr3qvASVHi9es", false)
            .add_routine_id(ROUTINE_ID)
            .send();
        assert!(matches!(sent, Ok(Some(_))));
        assert_eq!(accepted.records().len(), 1);
    }

    #[test]
    fn error_records_carry_trace_and_routine_ids_and_outcome_fields() {
        const ROUTINE_ID: &str = "ores-routine-8HNe_nBVinzuvJT9XjRsh";
        let transport = Arc::new(MemoryTransport::default());
        let log = Logger::new(options(LogLevel::Info).with_transport(transport.clone()));
        let sent = log
            .error(vec![json!("ores-otel command failed")])
            .add_fields(failure_fields(&CliError::Command("upstream".into())))
            .add_trace("ores-trace-MqRI6SOKnFOIhBgSQ56Vn", false)
            .add_routine_id(ROUTINE_ID)
            .send();
        assert!(matches!(sent, Ok(Some(_))));
        let records = transport.records();
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.app_name, APP_NAME);
        assert_eq!(
            record.trace_id.as_deref(),
            Some("ores-trace-MqRI6SOKnFOIhBgSQ56Vn")
        );
        assert_eq!(record.routine_id.as_deref(), Some(ROUTINE_ID));
        assert_eq!(record.fields.get("error.kind"), Some(&json!("command")));
        assert_eq!(record.fields.get("exit_code"), Some(&json!(1)));
    }
}
