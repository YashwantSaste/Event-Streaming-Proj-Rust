use crate::base::cli_error::CliError;
use crate::base::parsed_command::ParsedCommand;
use std::collections::HashMap;

pub struct CommandParser;

impl CommandParser {
    pub fn parse() -> Result<ParsedCommand, CliError> {
        let args: Vec<String> = std::env::args().skip(1).collect();

        if args.is_empty() {
            return Err(CliError::InvalidArguments("No command specified".into()));
        }

        let command = args[0].clone();
        let mut arguments = Vec::new();
        let mut options = HashMap::new();
        let mut index = 1;

        while index < args.len() {
            let current = &args[index];
            if current.starts_with("--") {
                let key = current.trim_start_matches("--");
                if index + 1 < args.len() && !args[index + 1].starts_with("--") {
                    options.insert(key.to_string(), args[index + 1].clone());
                    index += 2;
                } else {
                    options.insert(key.to_string(), "true".to_string());
                    index += 1;
                }
            } else {
                arguments.push(current.clone());
                index += 1;
            }
        }

        Ok(ParsedCommand {
            name: command,
            arguments,
            options,
        })
    }
}
