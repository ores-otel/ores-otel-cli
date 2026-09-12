#![forbid(unsafe_code)]

use next_loggers::Logger;
use serde_json::json;

use crate::config::Config;
use crate::error::CliError;
use crate::telemetry;

pub fn run(config: &Config, log: &Logger) -> Result<(), CliError> {
    const ROUTINE_ID: &str = "ores-routine-9EqI_5mTm8XoyNmhlHDu4";
    let body = serde_json::json!({
        "service": "ores-otel",
        "api_base": config.api_base,
    });
    if config.json {
        println!("{body}");
    } else {
        println!("ores-otel @ {}", config.api_base);
    }
    let _ = log
        .debug(vec![json!("status command completed")])
        .add_fields(telemetry::output_fields(config.json))
        .add_trace("ores-trace-cXvfa-aaHjP1Irt-Qfot8", false)
        .add_routine_id(ROUTINE_ID)
        .send();
    Ok(())
}
