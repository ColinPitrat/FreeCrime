use crate::resources::{Result, Error};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Mission {
    pub metadata: HashMap<String, String>,
    pub entries: Vec<MissionEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MissionEntry {
    Header(String, Vec<String>),
    Command {
        id: u32,
        permanent: bool,
        pos: Option<(f32, f32, f32)>,
        name: String,
        params: Vec<String>,
    },
}

pub fn parse_mission(content: &str) -> Result<Mission> {
    let cleaned = remove_comments(content);
    let mut mission = Mission::default();
    
    for line in cleaned.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        
        if line.starts_with('[') && line.ends_with(']') {
             // Level identifier or section?
             continue;
        }
        
        if let Ok(entry) = parse_line(line) {
            mission.entries.push(entry);
        }
    }
    
    Ok(mission)
}

fn remove_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_comment = 0;
    let mut chars = content.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '{' {
            in_comment += 1;
        } else if c == '}' {
            if in_comment > 0 { in_comment -= 1; }
        } else if in_comment == 0 {
            result.push(c);
        }
    }
    result
}

fn parse_line(line: &str) -> Result<MissionEntry> {
    // Basic tokenizer for the line
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_parens = false;
    let mut in_quotes = false;
    
    for c in line.chars() {
        if c == '(' {
            in_parens = true;
            current.push(c);
        } else if c == ')' {
            in_parens = false;
            current.push(c);
        } else if c == '"' {
            in_quotes = !in_quotes;
            current.push(c);
        } else if c.is_whitespace() && !in_parens && !in_quotes {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else if c == ',' && !in_parens && !in_quotes {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    
    if tokens.is_empty() {
        return Err(Error::Parse("Empty line".to_string()));
    }
    
    // Check if it's a command: starts with a number
    if let Ok(id) = tokens[0].parse::<u32>() {
        let mut idx = 1;
        let mut permanent = false;
        if idx < tokens.len() && tokens[idx] == "1" {
            permanent = true;
            idx += 1;
        }
        
        let mut pos = None;
        if idx < tokens.len() && tokens[idx].starts_with('(') {
            pos = parse_pos(&tokens[idx]);
            idx += 1;
        }
        
        if idx < tokens.len() {
            let name = tokens[idx].clone();
            let params = tokens[idx+1..].to_vec();
            return Ok(MissionEntry::Command { id, permanent, pos, name, params });
        }
    }
    
    Ok(MissionEntry::Header(tokens[0].clone(), tokens[1..].to_vec()))
}

fn parse_pos(s: &str) -> Option<(f32, f32, f32)> {
    let s = s.trim_matches(|c| c == '(' || c == ')');
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 3 {
        let x = parts[0].trim().parse().ok()?;
        let y = parts[1].trim().parse().ok()?;
        let z = parts[2].trim().parse().ok()?;
        return Some((x, y, z));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_comments() {
        let input = "Hello { comment } world { multi\nline }!";
        assert_eq!(remove_comments(input), "Hello  world !");
    }

    #[test]
    fn test_parse_line_command() {
        let line = "294 1 (105,119,4) PLAYER 293 256";
        let entry = parse_line(line).unwrap();
        if let MissionEntry::Command { id, permanent, pos, name, params } = entry {
            assert_eq!(id, 294);
            assert_eq!(permanent, true);
            assert_eq!(pos, Some((105.0, 119.0, 4.0)));
            assert_eq!(name, "PLAYER");
            assert_eq!(params, vec!["293", "256"]);
        } else {
            panic!("Expected command");
        }
    }
}
