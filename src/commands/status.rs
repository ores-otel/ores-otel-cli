#![forbid(unsafe_code)]

use crate::config::Config;
use crate::error::CliError;

pub fn run(config: &Config) -> Result<(), CliError> {
    let body = serde_json::json!({
        "service": "ores-otel",
        "api_base": config.api_base,
    });
    if config.json {
        println!("{body}");
    } else {
        println!("ores-otel @ {}", config.api_base);
    }
    Ok(())
}
