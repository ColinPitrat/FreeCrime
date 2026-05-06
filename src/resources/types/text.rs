use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TextBundle {
    pub entries: HashMap<String, String>,
}

impl Default for TextBundle {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBundle {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.entries.get(key)
    }
}
