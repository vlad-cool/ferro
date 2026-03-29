use std::vec;

use crate::syntax::{Connection, ConnectionDirection};

#[derive(Debug)]
pub struct ParseError {
    err_index: usize,
}

fn skip_ascii_whitespace<'a>(string: &'a str, offset: &mut usize) -> Result<(), ParseError> {
    let bytes: &[u8] = string.as_bytes();
    let mut i: usize = *offset;

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    if i >= bytes.len() {
        return Err(ParseError { err_index: i });
    }

    *offset = i;
    Ok(())
}

fn parse_word<'a>(string: &'a str, offset: &mut usize) -> Result<&'a str, ParseError> {
    skip_ascii_whitespace(string, offset)?;

    let bytes: &[u8] = string.as_bytes();
    let mut i: usize = *offset;
    let start: usize = i;
    let c: u8 = bytes[i];

    if !(c.is_ascii_alphabetic() || c == b'_') {
        return Err(ParseError { err_index: i });
    }
    i += 1;

    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_alphanumeric() || c == b'_' {
            i += 1;
        } else {
            break;
        }
    }

    *offset = i;

    Ok(&string[start..i])
}

fn parse_colon<'a>(string: &'a str, offset: &mut usize) -> Result<(), ParseError> {
    skip_ascii_whitespace(string, offset)?;

    let bytes: &[u8] = string.as_bytes();
    let mut i: usize = *offset;
    let c: u8 = bytes[i];

    if !(c == b':') {
        return Err(ParseError { err_index: i });
    }
    i += 1;

    *offset = i;
    Ok(())
}

fn parse_brackets<'a>(
    string: &'a str,
    open: char,
    close: char,
    offset: &mut usize,
) -> Result<&'a str, ParseError> {
    let bytes: &[u8] = string.as_bytes();
    let mut i: usize = *offset;

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    if i >= bytes.len() {
        eprintln!("Expectd (, got nothing");
        return Err(ParseError { err_index: i });
    }

    if bytes[i] != open as u8 {
        eprintln!("Expectd {}, got {}", open, bytes[i]);
        return Err(ParseError { err_index: i });
    }

    let start: usize = i;
    let mut count: i32 = 1;
    i += 1;

    while count > 0 && i < bytes.len() {
        if bytes[i] == open as u8 {
            count += 1;
        } else if bytes[i] == close as u8 {
            count -= 1;
        }
        i += 1;
    }

    if count == 0 {
        *offset = i;

        Ok(&string[start + 1..i - 1])
    } else {
        Err(ParseError { err_index: i })
    }
}

fn parse_interface<'a>(
    string: &'a str,
    offset: &mut usize,
) -> Result<Option<Connection>, ParseError> {
    if let Err(_) = skip_ascii_whitespace(string, offset) {
        return Ok(None);
    }

    let name: &str = parse_word(string, offset)?;
    parse_colon(string, offset)?;

    let direction: ConnectionDirection = match parse_word(string, offset)? {
        "input" => ConnectionDirection::Input,
        "output" => ConnectionDirection::Output,
        "inout" => ConnectionDirection::Inout,
        _ => {
            return Err(ParseError { err_index: *offset });
        }
    };

    skip_ascii_whitespace(string, offset)?;

    let bytes: &[u8] = string.as_bytes();

    let width: Option<String> = if *offset <= string.len() && bytes[*offset] == b'[' {
        Some(parse_brackets(string, '[', ']', offset)?.to_string())
    } else {
        None
    };

    if *offset <= string.len() && bytes[*offset] == b'<' {
        let modifiers: &str = parse_brackets(string, '<', '>', offset)?;

        println!("Name: {}; Modifiers: {}", name, modifiers);
    }

    skip_ascii_whitespace(string, offset)?;

    if *offset < bytes.len() && bytes[*offset] != b',' {
        return Err(ParseError { err_index: *offset });
    }

    if *offset < bytes.len() && bytes[*offset] == b',' {
        *offset += 1;
    }

    Ok(Some(Connection {
        name: name.to_string(),
        direction,
        width: width.unwrap_or("1".to_string()),
        modifiers: vec![],
    }))
}

fn parse_module<'a>(
    string: &'a str,
    offset: &mut usize,
) -> Result<(&'a str, &'a str, &'a str), ParseError> {
    let name: &str = parse_word(&string, offset)?;
    let interface: &str = parse_brackets(&string, '(', ')', offset)?;
    let body: &str = parse_brackets(&string, '{', '}', offset)?;
    Ok((name, interface, body))
}

pub fn parse_str(string: &str) -> Result<(), ParseError> {
    // TODO Remove comments before parsing, maybe added nested multiline comments?

    let mut offset: usize = 0;

    let keyword: &str = parse_word(string, &mut offset)?;

    match keyword {
        "module" => {
            let (name, interface, body) = (parse_module(string, &mut offset))?;
            eprintln!(
                "Module name: {} \n\n interface: {} \n\n body: {}",
                name, interface, body
            );

            let mut connections: Vec<Connection> = vec::Vec::<Connection>::new();

            let mut interface_offset: usize = 0;

            while let Some(connection) = parse_interface(interface, &mut interface_offset)? {
                println!("Connection: {:?}", connection);
                connections.push(connection);
            }
        }
        _ => {
            eprintln!("Unknown keyword {}", keyword);
            return Err(ParseError { err_index: offset });
        }
    }

    Ok(())
}
