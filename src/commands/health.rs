#![forbid(unsafe_code)]

use std::io;

use next_loggers::Logger;
use ores_clis_core::{paint, ColorRole, LogLevel, StreamEmitter};
use serde_json::json;

use crate::config::Config;
use crate::error::CliError;
use crate::telemetry;

pub fn run(config: &Config, log: &Logger) -> Result<(), CliError> {
    const ROUTINE_ID: &str = "ores-routine-r_N10yLRKeOPTlCak0jOd";
    let body = serde_json::json!({
        "ok": true,
        "api_base": config.api_base,
    });

    let stdout = io::stdout();
    let mut output = StreamEmitter::new(stdout.lock());
    if config.json {
        output
            .emit_json_line(&body.to_string())
            .map_err(|error| CliError::Command(format!("stdout write failed: {error}")))?;
    } else {
        let line = format!(
            "{} {}",
            paint(config.runtime.color_stdout(), ColorRole::Success, "ok"),
            paint(
                config.runtime.color_stdout(),
                ColorRole::Info,
                &config.api_base
            )
        );
        output
            .emit_line(&line)
            .map_err(|error| CliError::Command(format!("stdout write failed: {error}")))?;
    }

    if config.runtime.allows_log(LogLevel::Debug) {
        let _ = log
            .debug(vec![json!("health command completed")])
            .add_fields(telemetry::output_fields(config.json))
            .add_trace("ores-trace-L-LxAlnEnDJDjusvVcXla", false)
            .add_routine_id(ROUTINE_ID)
            .send();
    }
    Ok(())
}
