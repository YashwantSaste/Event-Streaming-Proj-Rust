use crate::command::command_factory::CommandFactory;
use crate::command::parser::CommandParser;

pub mod command;
pub mod base;
pub mod workspace;

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let parsed_command = CommandParser::parse()?;
    let command = CommandFactory::create(parsed_command)?;
    let result = command.execute()?;
    println!("{}", result.message);
    Ok(())
}