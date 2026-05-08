use crate::resources::Result;
use std::collections::HashMap;
use nom::{
    IResult, Parser,
    character::complete::{char, space0, space1, digit1, none_of},
    sequence::{delimited, preceded, terminated},
    multi::{separated_list0, many0},
    branch::alt,
    combinator::{map, opt, map_res, recognize},
};

/// Represents a parsed mission script or configuration file.
#[derive(Debug, Clone, Default)]
pub struct Mission {
    /// Key-value pairs extracted from the script (e.g. from section headers).
    pub metadata: HashMap<String, String>,
    /// Sequential list of script commands and sections.
    pub entries: Vec<MissionEntry>,
}

/// A single entry in a mission script.
#[derive(Debug, Clone, PartialEq)]
pub enum MissionEntry {
    /// A section header or non-command line (e.g. `[PLAYER_INFO]`).
    Header(String, Vec<String>),
    /// A gameplay command with an ID and optional parameters.
    Command {
        /// The unique numerical ID of the command.
        id: u32,
        /// True if the command is marked as "permanent" (prefixed by 1).
        permanent: bool,
        /// Optional 3D coordinates associated with the command.
        pos: Option<(f32, f32, f32)>,
        /// The name of the command (e.g. `PLAYER`, `OBJ`).
        name: String,
        /// List of string parameters for the command.
        params: Vec<String>,
    },
}

/// Parses a plaintext mission script (.INI or .SCR).
pub fn parse_mission(content: &str) -> Result<Mission> {
    let cleaned = remove_comments(content);
    let mut mission = Mission::default();

    for line in cleaned.lines() {
        let line = line.trim();
        // Skip standard INI section headers as they are currently handled as plain headers
        if line.is_empty() || (line.starts_with('[') && line.ends_with(']')) {
             continue;
        }

        if let Ok((_, entry)) = parse_line_nom(line) {
            mission.entries.push(entry);
        }
    }

    Ok(mission)
}

/// Removes curly-braced comments `{}` from script content, supporting nested blocks.
fn remove_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_comment = 0;
    let chars = content.chars();

    for c in chars {
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

/// Internal parser for floats in script files.
fn parse_f32(input: &str) -> IResult<&str, f32> {
    map_res(
        recognize((opt(char('-')), digit1, opt((char('.'), digit1)))),
        |s: &str| s.parse::<f32>()
    ).parse(input)
}

/// Internal parser for `(X,Y,Z)` coordinate triplets.
fn parse_pos_nom(input: &str) -> IResult<&str, (f32, f32, f32)> {
    delimited(
        char('('),
        (
            terminated_by_comma(parse_f32),
            terminated_by_comma(parse_f32),
            parse_f32
        ),
        char(')')
    ).parse(input)
}

/// Helper to parse a value optionally followed by a comma.
fn terminated_by_comma<'a, F, O>(mut inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where F: FnMut(&'a str) -> IResult<&'a str, O> {
    move |input| {
        let (input, out) = inner(input)?;
        let (input, _) = opt(char(',')).parse(input)?;
        Ok((input, out))
    }
}

/// Internal parser for space- or comma-separated tokens, supporting quotes.
fn parse_token(input: &str) -> IResult<&str, String> {
    map(
        alt((
            delimited(char('"'), recognize(many0(none_of("\""))), char('"')),
            recognize(many0(none_of(" ,\t\n\r()")))
        )),
        |s: &str| s.to_string()
    ).parse(input)
}

/// Primary line parser for the mission script format.
fn parse_line_nom(input: &str) -> IResult<&str, MissionEntry> {
    let (input, id_str) = recognize(digit1).parse(input)?;
    let (input, _) = space0(input)?;

    if let Ok(id) = id_str.parse::<u32>() {
        // Try Command format: ID [1] [(X,Y,Z)] NAME [PARAMS...]
        let (input, permanent) = map(opt((char('1'), space1)), |o| o.is_some()).parse(input)?;
        let (input, pos) = opt(terminated(parse_pos_nom, space0)).parse(input)?;
        let (input, name) = parse_token(input)?;
        let (input, params) = separated_list0(alt((char(','), char(' '), char('\t'))), preceded(space0, parse_token)).parse(input)?;

        let params = params.into_iter().filter(|s: &String| !s.is_empty()).collect();
        return Ok((input, MissionEntry::Command { id, permanent, pos, name, params }));
    }

    // Header format: NAME [PARAMS...]
    let (input, name) = parse_token(id_str)?;
    let (input, params) = separated_list0(alt((char(','), char(' '), char('\t'))), preceded(space0, parse_token)).parse(input)?;
    let params = params.into_iter().filter(|s: &String| !s.is_empty()).collect();

    Ok((input, MissionEntry::Header(name, params)))
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
        let (_, entry) = parse_line_nom(line).unwrap();
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
