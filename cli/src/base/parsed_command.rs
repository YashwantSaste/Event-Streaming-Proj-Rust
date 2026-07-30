use std::collections::HashMap;

pub struct ParsedCommand {
    pub name: String,
    pub arguments: Vec<String>,
    pub options: HashMap<String, String>,
}

impl ParsedCommand {
    pub fn argument(&self, index: usize) -> Option<&str> {
        self.arguments.get(index).map(|s| s.as_str())
    }

    pub fn option(&self, key: &str) -> Option<&str> {
        self.options.get(key).map(|s| s.as_str())
    }

    pub fn has_option(&self, key: &str) -> bool {
        self.options.contains_key(key)
    }
}
