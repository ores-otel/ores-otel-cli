#![forbid(unsafe_code)]

use next_loggers::Logger;
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
    if config.json {
        println!("{body}");
    } else {
        println!("ok {}", config.api_base);
    }
    let _ = log
        .debug(vec![json!("health command completed")])
        .add_fields(telemetry::output_fields(config.json))
        .add_trace("ores-trace-L-LxAlnEnDJDjusvVcXla", false)
        .add_routine_id(ROUTINE_ID)
        .send();
    Ok(())
}
