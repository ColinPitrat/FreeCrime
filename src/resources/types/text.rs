use std::collections::HashMap;

/// Represents a collection of text strings (FXT) keyed by ID or name.
#[derive(Debug, Clone, PartialEq)]
pub struct TextBundle {
    /// The string entries.
    pub entries: HashMap<String, String>,
}

impl Default for TextBundle {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBundle {
    /// Creates an empty text bundle.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Retrieves a string entry by its key.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.entries.get(key)
    }
}
