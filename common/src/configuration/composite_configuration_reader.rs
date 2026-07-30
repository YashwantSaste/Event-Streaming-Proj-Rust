use crate::configuration::configuration::Configuration;
use crate::configuration::configuration_reader::ConfigurationReader;
use crate::error::configuration_error::ConfigurationError;

pub struct CompositeConfigurationReader {
    readers: Vec<Box<dyn ConfigurationReader>>,
}

impl CompositeConfigurationReader {
    pub fn new(readers: Vec<Box<dyn ConfigurationReader>>) -> Self {
        Self { readers }
    }
}

impl ConfigurationReader for CompositeConfigurationReader {
    fn read(&self) -> Result<Configuration, ConfigurationError> {
        self.readers
            .iter()
            .try_fold(Configuration::empty(), |mut configuration, reader| {
                configuration.merge(reader.read()?);
                Ok(configuration)
            })
    }
}
