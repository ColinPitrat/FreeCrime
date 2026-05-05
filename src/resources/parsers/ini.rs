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
        if line.is_empty() || (line.starts_with('[') && line.ends_with(']')) {
             continue;
        }

        if let Ok((_, entry)) = parse_line_nom(line) {
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

fn parse_f32(input: &str) -> IResult<&str, f32> {
    map_res(
        recognize((opt(char('-')), digit1, opt((char('.'), digit1)))),
        |s: &str| s.parse::<f32>()
    ).parse(input)
}

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

fn terminated_by_comma<'a, F, O>(mut inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where F: FnMut(&'a str) -> IResult<&'a str, O> {
    move |input| {
        let (input, out) = inner(input)?;
        let (input, _) = opt(char(',')).parse(input)?;
        Ok((input, out))
    }
}

fn parse_token(input: &str) -> IResult<&str, String> {
    map(
        alt((
            delimited(char('"'), recognize(many0(none_of("\""))), char('"')),
            recognize(many0(none_of(" ,\t\n\r()")))
        )),
        |s: &str| s.to_string()
    ).parse(input)
}

fn parse_line_nom(input: &str) -> IResult<&str, MissionEntry> {
    let (input, id_str) = recognize(digit1).parse(input)?;
    let (input, _) = space0(input)?;

    if let Ok(id) = id_str.parse::<u32>() {
        // Try Command
        let (input, permanent) = map(opt((char('1'), space1)), |o| o.is_some()).parse(input)?;
        let (input, pos) = opt(terminated(parse_pos_nom, space0)).parse(input)?;
        let (input, name) = parse_token(input)?;
        let (input, params) = separated_list0(alt((char(','), char(' '), char('\t'))), preceded(space0, parse_token)).parse(input)?;

        let params = params.into_iter().filter(|s: &String| !s.is_empty()).collect();
        return Ok((input, MissionEntry::Command { id, permanent, pos, name, params }));
    }

    // Header
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
