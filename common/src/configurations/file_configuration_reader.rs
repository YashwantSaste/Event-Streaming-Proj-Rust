pub struct FileConfiguration {
    file_path: String,
}

impl FileConfiguration {
    pub fn new(file_path: &str) -> FileConfiguration {
        Self {
            file_path: file_path.into(),
        }
    }
}

impl ConfigurationReader<T> for FileConfiguration {

    type Error = ConfigurationError;

    fn read(&self) -> Result<T, ConfigurationError> {
        todo!()
    }
}