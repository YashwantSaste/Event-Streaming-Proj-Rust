use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Configuration {
    values: HashMap<String, String>,
}

impl Configuration {
    pub fn new(values: HashMap<String, String>) -> Self {
        Self { values }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    pub fn merge(&mut self, other: Configuration) {
        self.values.extend(other.values);
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn values(&self) -> &HashMap<String, String> {
        &self.values
    }
}
