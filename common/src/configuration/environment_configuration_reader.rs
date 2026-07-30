use std::collections::HashMap;

use crate::configuration::configuration::Configuration;
use crate::configuration::configuration_reader::ConfigurationReader;
use crate::error::configuration_error::ConfigurationError;

pub struct EnvironmentConfigurationReader {
    prefix: Option<String>,
}

impl EnvironmentConfigurationReader {
    pub fn new(prefix: Option<String>) -> Self {
        Self { prefix }
    }

    fn normalize_key(&self, key: String) -> Option<String> {
        match &self.prefix {
            Some(prefix) => key
                .strip_prefix(prefix)
                .map(|stripped| stripped.trim_start_matches('_').to_ascii_lowercase()),
            None => Some(key.to_ascii_lowercase()),
        }
    }
}

impl ConfigurationReader for EnvironmentConfigurationReader {
    fn read(&self) -> Result<Configuration, ConfigurationError> {
        let values = std::env::vars()
            .filter_map(|(key, value)| self.normalize_key(key).map(|key| (key, value)))
            .filter(|(key, _)| !key.is_empty())
            .collect::<HashMap<_, _>>();

        Ok(Configuration::new(values))
    }
}
