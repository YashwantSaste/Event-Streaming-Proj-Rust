use crate::configuration::configuration::Configuration;
use crate::error::configuration_error::ConfigurationError;

pub trait ConfigurationReader {
    fn read(&self) -> Result<Configuration, ConfigurationError>;
}
