#![forbid(unsafe_code)]

use crate::error::CliError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Health,
    Status,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub command: Command,
    pub api_base: Option<String>,
    pub json: bool,
}

pub fn parse<I>(args: I) -> Result<Invocation, CliError>
where
    I: IntoIterator<Item = String>,
{
    let mut command = Command::Help;
    let mut api_base = None;
    let mut json = false;
    let mut items = args.into_iter().peekable();
    if let Some(first) = items.peek() {
        match first.as_str() {
            "health" => {
                command = Command::Health;
                items.next();
            }
            "status" => {
                command = Command::Status;
                items.next();
            }
            "-h" | "--help" | "help" => {
                command = Command::Help;
                items.next();
            }
            other if !other.starts_with('-') => {
                return Err(CliError::Usage("unknown command".into()));
            }
            _ => {}
        }
    }
    for arg in items {
        match arg.as_str() {
            "--json" => json = true,
            "--help" | "-h" => command = Command::Help,
            flag if flag.starts_with("--api-base=") => {
                api_base = Some(flag.trim_start_matches("--api-base=").to_string());
            }
            _ => return Err(CliError::Usage("unknown flag".into())),
        }
    }
    Ok(Invocation {
        command,
        api_base,
        json,
    })
}

pub fn help_text() -> &'static str {
    "ores-otel — ores-otel CLI\n\nCommands:\n  health\n  status\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_argument_parser_never_repeats_untrusted_input() {
        for argv in [
            vec!["token=secret-value".to_owned()],
            vec!["health".to_owned(), "--token=secret-value".to_owned()],
        ] {
            let error = parse(argv).expect_err("unrecognized argument must fail").to_string();
            assert!(!error.contains("secret-value"));
        }
    }
}
