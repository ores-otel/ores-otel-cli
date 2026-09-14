#![forbid(unsafe_code)]

#[path = "../generated/rust/env.rs"]
mod env;

use ores_clis_core::{
    CliPolicy, ColorMode, EnvironmentHints, LogLevel, OutputMode, RuntimePolicy, TerminalState,
};

use crate::env_map::{truthy, value, EnvMap};
use crate::error::CliError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeContext {
    pub terminal: TerminalState,
    pub environment: EnvironmentHints,
}

impl RuntimeContext {
    #[must_use]
    pub fn detect() -> Self {
        Self {
            terminal: TerminalState::detect(),
            environment: EnvironmentHints::detect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub api_base: String,
    pub json: bool,
    pub runtime: RuntimePolicy,
}

impl Config {
    pub fn from_env_map(env_map: &EnvMap) -> Result<Self, CliError> {
        Self::from_env_map_with_context(env_map, RuntimeContext::detect())
    }

    pub fn from_env_map_with_context(
        env_map: &EnvMap,
        context: RuntimeContext,
    ) -> Result<Self, CliError> {
        let api_base = value(env_map, env::API_BASE)
            .unwrap_or("http://127.0.0.1:8080")
            .to_owned();
        if api_base.trim().is_empty() {
            return Err(CliError::Config("API base is empty".into()));
        }

        let output = match value(env_map, env::JSON) {
            Some(_) if truthy(env_map, env::JSON) => OutputMode::Json,
            Some(_) => OutputMode::Human,
            None => OutputMode::Auto,
        };
        let color = match value(env_map, env::COLOR) {
            Some(_) if truthy(env_map, env::COLOR) => ColorMode::Always,
            Some(_) => ColorMode::Never,
            None => ColorMode::Auto,
        };
        let log_level = value(env_map, env::LOG_LEVEL)
            .unwrap_or(env::LOG_LEVEL_DEFAULT)
            .parse::<LogLevel>()
            .map_err(|error| CliError::Config(error.to_string()))?;

        let runtime = CliPolicy {
            output,
            color,
            log_level,
        }
        .resolve(context.terminal, context.environment);

        Ok(Self {
            api_base,
            json: runtime.json(),
            runtime,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(stdout_tty: bool, stderr_tty: bool) -> RuntimeContext {
        RuntimeContext {
            terminal: TerminalState {
                stdin_tty: false,
                stdout_tty,
                stderr_tty,
            },
            environment: EnvironmentHints::default(),
        }
    }

    #[test]
    fn output_defaults_to_human_on_tty() {
        let config =
            Config::from_env_map_with_context(&EnvMap::new(), context(true, true)).unwrap();
        assert!(!config.json);
        assert!(config.runtime.color_stdout());
    }

    #[test]
    fn output_defaults_to_json_when_stdout_is_not_tty() {
        let config =
            Config::from_env_map_with_context(&EnvMap::new(), context(false, false)).unwrap();
        assert!(config.json);
        assert!(!config.runtime.color_stdout());
    }

    #[test]
    fn explicit_no_json_and_no_color_override_tty_defaults() {
        let env_map = EnvMap::from([
            (env::JSON.to_owned(), "false".to_owned()),
            (env::COLOR.to_owned(), "false".to_owned()),
        ]);
        let config = Config::from_env_map_with_context(&env_map, context(false, true)).unwrap();
        assert!(!config.json);
        assert!(!config.runtime.color_stdout());
        assert!(!config.runtime.color_stderr());
    }

    #[test]
    fn trace_log_level_is_supported() {
        let env_map = EnvMap::from([(env::LOG_LEVEL.to_owned(), "trace".to_owned())]);
        let config = Config::from_env_map_with_context(&env_map, context(true, true)).unwrap();
        assert_eq!(config.runtime.log_level(), LogLevel::Trace);
    }
}
