#![forbid(unsafe_code)]

use ores_clis_core::LogLevel;
use ores_otel_cli::{args, commands, config, error::CliError, flags, telemetry};
use serde_json::json;

fn main() {
    if let Err(err) = run() {
        // Primary error output is not a diagnostic log record and therefore is
        // never hidden by --log-level=silent/quiet.
        eprintln!("{err}");
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
        print!("{}", args::help_text());
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
