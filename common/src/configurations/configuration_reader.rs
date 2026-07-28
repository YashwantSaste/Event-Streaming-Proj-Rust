pub trait ConfigurationReader<T> {

    type Error : ConfigurationError;

    fn read(&self) -> Result<T, Self::Error>;
}