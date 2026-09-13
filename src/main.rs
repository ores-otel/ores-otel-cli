#![forbid(unsafe_code)]

use next_loggers::Logger;
use ores_otel_cli::{args, commands, config, error::CliError, flags, telemetry};
use serde_json::json;

fn main() {
    const ROUTINE_ID: &str = "ores-routine-raqKUTKQkefHuQ2fCTbku";
    let log = telemetry::logger();
    if let Err(err) = run(&log) {
        // Telemetry never changes the command's outcome, so a failed send is ignored.
        let _ = log
            .error(vec![json!("ores-otel command failed")])
            .add_fields(telemetry::failure_fields(&err))
            .add_trace("ores-trace-bWZsJe8VWrlKarMIY9W0s", false)
            .add_routine_id(ROUTINE_ID)
            .send();
        eprintln!("{err}");
        std::process::exit(err.exit_code());
    }
}

fn run(log: &Logger) -> Result<(), CliError> {
    const ROUTINE_ID: &str = "ores-routine-4fvqUUxzRsmlq0b1NEP6y";
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
    let _ = log
        .debug(vec![json!("ores-otel command dispatched")])
        .add_fields(telemetry::command_fields(&command))
        .add_trace("ores-trace-0l1yyN_WDIHql1ypEjEIr", false)
        .add_routine_id(ROUTINE_ID)
        .send();
    commands::dispatch(&cfg, command, log)
}
