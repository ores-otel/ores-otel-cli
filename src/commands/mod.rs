#![forbid(unsafe_code)]

pub mod health;
pub mod status;

use next_loggers::Logger;

use crate::args::Command;
use crate::config::Config;
use crate::error::CliError;

pub fn dispatch(config: &Config, command: Command, log: &Logger) -> Result<(), CliError> {
    match command {
        Command::Help => {
            print!("{}", crate::args::help_text());
            Ok(())
        }
        Command::Health => health::run(config, log),
        Command::Status => status::run(config, log),
    }
}
