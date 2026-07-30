use std::path::PathBuf;

use crate::configuration::configuration::Configuration;
use crate::configuration::configuration_reader::ConfigurationReader;
use crate::error::configuration_error::ConfigurationError;
use crate::filesystem::file_system::FileSystem;

pub struct TomlConfigurationReader<F>
where
    F: FileSystem,
{
    file_system: F,
    path: PathBuf,
}

impl<F> TomlConfigurationReader<F>
where
    F: FileSystem,
{
    pub fn new(file_system: F, path: PathBuf) -> Self {
        Self { file_system, path }
    }

    fn parse(content: &str) -> Result<Configuration, ConfigurationError> {
        let mut configuration = Configuration::empty();
        let mut current_section: Option<String> = None;

        for (line_number, raw_line) in content.lines().enumerate() {
            let line = Self::strip_comment(raw_line).trim().to_string();

            if line.is_empty() {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = Some(Self::parse_section(&line, line_number)?);
                continue;
            }

            let (key, value) = Self::parse_key_value(&line, line_number)?;
            let full_key = match &current_section {
                Some(section) => format!("{section}.{key}"),
                None => key,
            };

            configuration.insert(full_key, value);
        }

        Ok(configuration)
    }

    fn strip_comment(line: &str) -> &str {
        match line.find('#') {
            Some(index) => &line[..index],
            None => line,
        }
    }

    fn parse_section(line: &str, line_number: usize) -> Result<String, ConfigurationError> {
        let section = line.trim_start_matches('[').trim_end_matches(']').trim();

        if section.is_empty() {
            return Err(Self::invalid_line(
                line_number,
                "section name cannot be empty",
            ));
        }

        Ok(section.to_string())
    }

    fn parse_key_value(
        line: &str,
        line_number: usize,
    ) -> Result<(String, String), ConfigurationError> {
        let Some((key, value)) = line.split_once('=') else {
            return Err(Self::invalid_line(line_number, "expected key = value"));
        };

        let key = key.trim();
        if key.is_empty() {
            return Err(Self::invalid_line(line_number, "key cannot be empty"));
        }

        Ok((key.to_string(), Self::normalize_value(value.trim())))
    }

    fn normalize_value(value: &str) -> String {
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            value[1..value.len() - 1].to_string()
        } else {
            value.to_string()
        }
    }

    fn invalid_line(line_number: usize, reason: &str) -> ConfigurationError {
        ConfigurationError::new(format!(
            "Invalid TOML configuration at line {}: {reason}",
            line_number + 1
        ))
    }
}

impl<F> ConfigurationReader for TomlConfigurationReader<F>
where
    F: FileSystem,
{
    fn read(&self) -> Result<Configuration, ConfigurationError> {
        let bytes = self
            .file_system
            .read_file(&self.path)
            .map_err(|error| ConfigurationError::new(error.to_string()))?;

        let content = String::from_utf8(bytes).map_err(|error| {
            ConfigurationError::new(format!("Configuration is not UTF-8: {error}"))
        })?;

        Self::parse(&content)
    }
}
