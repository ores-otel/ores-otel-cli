#![forbid(unsafe_code)]

use std::io;

use ores_clis_core::{
    EmitDisposition, LogLevel, ProtocolEmitter, StreamRole, top_level_io,
};
use ores_otel_cli::{args, commands, config, error::CliError, flags, telemetry};
use serde_json::json;

fn main() {
    if let Err(err) = run() {
        // Primary command failure reporting remains non-suppressible for
        // compatibility, but it still owns the diagnostics/stderr stream and
        // receives the shared BrokenPipe classification.
        emit_top_level_error(&err);
        std::process::exit(err.exit_code());
    }
}

fn run() -> Result<(), CliError> {
    const DISPATCH_ROUTINE_ID: &str = "ores-routine-4fvqUUxzRsmlq0b1NEP6y";
    const FAILURE_ROUTINE_ID: &str = "ores-routine-raqKUTKQkefHuQ2fCTbku";

    let argv = std::env::args().collect::<Vec<_>>();
    if argv
        .iter()
        .skip(1)
        .any(|argument| matches!(argument.as_str(), "-h" | "--help" | "help"))
    {
        emit_help(args::help_text())?;
        return Ok(());
    }

    let (command, env) = flags::apply_cli_flags()?;
    let cfg = config::Config::from_env_map(&env)?;
    let log = telemetry::logger();

    if cfg.runtime.allows_log(LogLevel::Debug) {
        let _ = log
            .debug(vec![json!("ores-otel command dispatched")])
            .add_fields(telemetry::command_fields(&command))
            .add_trace("ores-trace-0l1yyN_WDIHql1ypEjEIr", false)
            .add_routine_id(DISPATCH_ROUTINE_ID)
            .send();
    }

    match commands::dispatch(&cfg, command, &log) {
        Ok(()) => Ok(()),
        Err(err) => {
            // Telemetry never changes the command outcome. Shared runtime
            // policy is the single authority deciding whether this record is
            // eligible; the backend is deliberately permissive enough to send it.
            if cfg.runtime.allows_log(LogLevel::Error) {
                let _ = log
                    .error(vec![json!("ores-otel command failed")])
                    .add_fields(telemetry::failure_fields(&err))
                    .add_trace("ores-trace-bWZsJe8VWrlKarMIY9W0s", false)
                    .add_routine_id(FAILURE_ROUTINE_ID)
                    .send();
            }
            Err(err)
        }
    }
}

fn emit_help(value: &str) -> Result<(), CliError> {
    let stdout = io::stdout();
    let mut emitter = ProtocolEmitter::new(stdout.lock(), StreamRole::Primary);
    match top_level_io(emitter.emit_primary_human_line(value.trim_end_matches('\n')))
        .map_err(|error| CliError::Command(format!("could not write help output: {error}")))?
    {
        EmitDisposition::Written | EmitDisposition::ConsumerClosed => Ok(()),
    }
}

fn emit_top_level_error(error: &CliError) {
    let stderr = io::stderr();
    let mut emitter = ProtocolEmitter::new(stderr.lock(), StreamRole::Diagnostics);
    let _ = top_level_io(emitter.emit_diagnostic_line(&error.to_string()));
}
