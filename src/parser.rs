use std::{rc::Rc, vec};

use crate::{
    expression::{
        BinaryExpression, BinaryOperation, CompileTimeExpression, Const, Parameter, UnaryExpression,
    },
    syntax::{Port, PortDir},
};

#[derive(Debug)]
pub struct ParseError {
    err_index: usize,
}

fn skip_ascii_whitespace(string: &str, offset: &mut usize) -> Result<(), ParseError> {
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

fn is_empty(string: &str, offset: &mut usize) -> bool {
    *offset < string.len()
}

fn parse_keyword(string: &str, offset: &mut usize, keyword: &str) -> bool {
    if *offset + keyword.len() > string.len() {
        return false;
    }
    if string[*offset..].starts_with(keyword) {
        if *offset + keyword.len() == string.len() {
            true
        } else {
            let char: u8 = string[*offset + keyword.len()..].as_bytes()[0];

            if !char.is_ascii_alphanumeric() && !(char == b'_') {
                *offset += keyword.len();
                true
            } else {
                false
            }
        }
    } else {
        false
    }
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

// TODO add support for hex / bin / oct / dec in format NfM, where N is width, f is base and M is number
fn parse_number(string: &str, offset: &mut usize) -> Result<usize, ParseError> {
    skip_ascii_whitespace(string, offset)?;

    let bytes: &[u8] = string.as_bytes();
    let mut i: usize = *offset;
    let start: usize = i;
    let c: u8 = bytes[i];

    if !(c.is_ascii_digit()) {
        return Err(ParseError { err_index: i });
    }
    i += 1;

    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_digit() {
            i += 1;
        } else {
            break;
        }
    }

    *offset = i;

    Ok(string[start..i].parse::<usize>().unwrap())
}

fn parse_colon(string: &str, offset: &mut usize) -> Result<(), ParseError> {
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

fn parse_port_list<'a>(string: &'a str, offset: &mut usize) -> Result<Option<Port>, ParseError> {
    if let Err(_) = skip_ascii_whitespace(string, offset) {
        return Ok(None);
    }

    let name: &str = parse_word(string, offset)?;
    parse_colon(string, offset)?;

    let direction: PortDir = match parse_word(string, offset)? {
        "input" => PortDir::Input,
        "output" => PortDir::Output,
        "inout" => PortDir::Inout,
        _ => {
            return Err(ParseError { err_index: *offset });
        }
    };

    skip_ascii_whitespace(string, offset)?;

    let bytes: &[u8] = string.as_bytes();

    let width: Rc<Box<dyn CompileTimeExpression>> = if *offset <= string.len()
        && bytes[*offset] == b'['
    {
        let mut inner_offset: usize = 0;
        parse_compile_time_expression(parse_brackets(string, '[', ']', offset)?, &mut inner_offset)?
    } else {
        Rc::new(Box::new(Const { value: 1 }))
    };

    if *offset <= string.len() && bytes[*offset] == b'<' {
        let modifiers: &str = parse_brackets(string, '<', '>', offset)?;

        println!("Name: {}; Modifiers: {}", name, modifiers);
    }

    if let Err(_) = skip_ascii_whitespace(string, offset) {
        return Ok(None);
    }

    if *offset < bytes.len() && bytes[*offset] != b',' {
        return Err(ParseError { err_index: *offset });
    }

    if *offset < bytes.len() && bytes[*offset] == b',' {
        *offset += 1;
    }

    Ok(Some(Port {
        name: name.to_string(),
        direction,
        width,
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

pub fn parse_inner_expression(
    string: &str,
    offset: &mut usize,
) -> Result<Rc<Box<dyn CompileTimeExpression>>, ParseError> {
    let inner_string: &str = parse_brackets(string, '(', ')', offset)?;
    eprintln!("Inner string: {}", inner_string);
    let mut inner_offset: usize = 0;

    match parse_compile_time_expression(inner_string, &mut inner_offset) {
        Ok(exp) => Ok(exp),
        Err(err) => Err(ParseError {
            err_index: err.err_index + *offset,
        }),
    }
}

fn collapse_expression(
    expressions: &mut Vec<Rc<Box<dyn CompileTimeExpression>>>,
    operations: &mut Vec<BinaryOperation>,
) -> Rc<Box<dyn CompileTimeExpression>> {
    if expressions.len() == 0 {
        panic!("Got 0 expressions to collapse") // TODO
    } else if expressions.len() == 1 {
        return expressions[0].clone();
    } else {
        let max_priority: u32 = operations.iter().map(|op| op.priority()).min().unwrap();

        for i in 0..operations.len() {
            if operations[i].priority() == max_priority {
                expressions[i] = Rc::new(Box::new(BinaryExpression {
                    lhs: expressions[i].clone(),
                    rhs: expressions[i + 1].clone(),
                    op: operations[i].clone(),
                }));
                expressions.remove(i + 1);
                operations.remove(i);
                return collapse_expression(expressions, operations);
            }
        }

        panic!()
    }
}

pub fn parse_compile_time_expression(
    string: &str,
    offset: &mut usize,
) -> Result<Rc<Box<dyn CompileTimeExpression>>, ParseError> {
    skip_ascii_whitespace(string, offset)?;
    // is_empty(string, offset)?;

    let mut expressions: Vec<Rc<Box<dyn CompileTimeExpression>>> = vec![];
    let mut operations: Vec<BinaryOperation> = vec![];

    while *offset < string.len() {
        let first_char = string.as_bytes()[*offset];

        if first_char.is_ascii_whitespace() {
            *offset = *offset + 1;
        } else if first_char == b'(' {
            eprintln!("Offset before: {}", *offset);
            expressions.push(parse_inner_expression(string, offset)?);
            eprintln!("Offset after: {}", *offset);
        } else if first_char == b'+' {
            *offset = *offset + 1;
            operations.push(BinaryOperation::Add);
        } else if first_char == b'-' {
            *offset = *offset + 1;
            operations.push(BinaryOperation::Sub);
        } else if first_char == b'*' {
            *offset = *offset + 1;
            operations.push(BinaryOperation::Mul);
        } else if first_char == b'/' {
            *offset = *offset + 1;
            operations.push(BinaryOperation::Div);
        } else if first_char == b'%' {
            *offset = *offset + 1;
            operations.push(BinaryOperation::Mod);
        } else if parse_keyword(string, offset, "clog2") {
            expressions.push(Rc::new(Box::new(UnaryExpression {
                exp: parse_inner_expression(string, offset)?,
                op: crate::expression::UnaryOperation::Clog2,
            })));
        } else if parse_keyword(string, offset, "max") {
            let lhs: Rc<Box<dyn CompileTimeExpression>> = parse_inner_expression(string, offset)?;
            parse_colon(string, offset)?;
            let rhs: Rc<Box<dyn CompileTimeExpression>> = parse_inner_expression(string, offset)?;

            expressions.push(Rc::new(Box::new(BinaryExpression {
                lhs,
                rhs,
                op: crate::expression::BinaryOperation::Max,
            })));
        } else if parse_keyword(string, offset, "min") {
            let lhs: Rc<Box<dyn CompileTimeExpression>> = parse_inner_expression(string, offset)?;
            parse_colon(string, offset)?;
            let rhs: Rc<Box<dyn CompileTimeExpression>> = parse_inner_expression(string, offset)?;

            expressions.push(Rc::new(Box::new(BinaryExpression {
                lhs,
                rhs,
                op: crate::expression::BinaryOperation::Min,
            })));
        } else if let Ok(number) = parse_number(string, offset) {
            expressions.push(Rc::new(Box::new(Const { value: number })));
        } else if let Ok(name) = parse_word(string, offset) {
            expressions.push(Rc::new(Box::new(Parameter { name: name.into() })));
        } else {
            *offset = *offset + 1;
            eprintln!("Got unexpected char: {}", first_char as char); // TODO proper error propagation
        }
    }

    assert!(expressions.len() > 0); // TODO

    eprintln!("Expressions: {:?}", expressions);
    eprintln!("Operations: {:?}", operations);

    Ok(collapse_expression(&mut expressions, &mut operations))
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

            let mut connections: Vec<Port> = vec::Vec::<Port>::new();

            let mut interface_offset: usize = 0;

            while let Some(connection) = parse_port_list(interface, &mut interface_offset)? {
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

#[cfg(test)]
mod tests {
    use rand::{self, RngExt};
    use std::collections::HashMap;

    use super::*;

    fn clog2(n: usize) -> usize {
        let mut res: usize = 0;
        let mut value: usize = n;

        while value > 0 {
            value /= 2;
            res += 1;
        }
        res
    }

    #[test]
    fn test_parse_brackets() {
        let test_str: [&str; _] = [
            "()",
            "(1)",
            "(x)",
            "(clog2(8))",
            "(1+2)",
            "(x+y)",
            "((1+2))",
            "((x+y))",
            "(1+2*3)",
            "((1+2)*3)",
            "(1+(2*3))",
            "(((1)))",
            "(((x)))",
            "(clog2((8)))",
            "(x+(y*z))",
            "((x+1)+(y+2))",
            "((a+b)*(c+d))",
            "((1+clog2(8))*2)",
            "((x*y)+(z/3))",
            "((a+b)+((c+d)*e))",
            "(1+2+(3+4))",
            "((x+y)+(z+1))",
            "(clog2(x+y))",
            "((1+2)*(3+4))",
            "(((1+2)*3)+4)",
            "(x+((y+z)*2))",
            "((clog2(x)+clog2(y))*z)",
            "((1+2)*(x+3))",
            "((a*b)+(c*d))",
            "(((x+y)+z))",
            "(((1+2)+(3+4)))",
            "((x+(y+z))*2)",
            "(((x)))",
            "(x+(y*clog2(8)))",
            "((1+clog2(4))+(x*2))",
            "(((a*b)+c))",
            "((x+y+z))",
            "(((1+2)*(3+4)))",
            "(x+((y+z)+clog2(8)))",
            "((clog2(2*x)+y)-z)",
            "((x*y)+(clog2(y+z)))",
            "((x+(y*z))+((a+b)*c))",
            "(((x*y)+(z/2)))",
            "((x*2)+((y+3)*z))",
            "(((x+1)+y)+z)",
            "((1+(2*(3+4))))",
            "((x+((y+z)+1)))",
            "((x+(y+(z*2))))",
        ];

        let ref_str: [&str; _] = [
            "",
            "1",
            "x",
            "clog2(8)",
            "1+2",
            "x+y",
            "(1+2)",
            "(x+y)",
            "1+2*3",
            "(1+2)*3",
            "1+(2*3)",
            "((1))",
            "((x))",
            "clog2((8))",
            "x+(y*z)",
            "(x+1)+(y+2)",
            "(a+b)*(c+d)",
            "(1+clog2(8))*2",
            "(x*y)+(z/3)",
            "(a+b)+((c+d)*e)",
            "1+2+(3+4)",
            "(x+y)+(z+1)",
            "clog2(x+y)",
            "(1+2)*(3+4)",
            "((1+2)*3)+4",
            "x+((y+z)*2)",
            "(clog2(x)+clog2(y))*z",
            "(1+2)*(x+3)",
            "(a*b)+(c*d)",
            "((x+y)+z)",
            "((1+2)+(3+4))",
            "(x+(y+z))*2",
            "((x))",
            "x+(y*clog2(8))",
            "(1+clog2(4))+(x*2)",
            "((a*b)+c)",
            "(x+y+z)",
            "((1+2)*(3+4))",
            "x+((y+z)+clog2(8))",
            "(clog2(2*x)+y)-z",
            "(x*y)+(clog2(y+z))",
            "(x+(y*z))+((a+b)*c)",
            "((x*y)+(z/2))",
            "(x*2)+((y+3)*z)",
            "((x+1)+y)+z",
            "(1+(2*(3+4)))",
            "(x+((y+z)+1))",
            "(x+(y+(z*2)))",
        ];

        for i in 0..test_str.len() {
            let mut offset: usize = 0;

            eprintln!("Iteration: {}", i);

            assert_eq!(
                parse_brackets(test_str[i], '(', ')', &mut offset).unwrap(),
                ref_str[i]
            );
            assert_eq!(offset, test_str[i].len());
        }
    }

    #[test]
    fn test_parse_expressions() {
        let test_str: [&str; 48] = [
            "0",
            "1",
            "42",
            "123",
            "clog2(1)",
            "clog2(8)",
            "1+2",
            "4-3",
            "5*6",
            "7/8",
            "9%2",
            "1+2+3",
            "4*5-6",
            "7+8*9",
            "(1+2)*3",
            "4*(5+6)",
            "clog2   (   16)+   2",
            "3+clog2(8)  ",
            "clog2(  4  *  4)",
            "(2+3)  *(4 + 5)",
            "1+2*3-4",
            "clog2(32)-   5",
            "(10/2)+6",
            "(8%3)+clog2   (8)",
            "1+2+3   +4",
            "5*6*7  ",
            "8-3+2",
            "(1+1 )*(2+2)",
            "clog2(2)+clog2(8)",
            "(1+2+3)*4",
            "(5*6)-(7+8)",
            "clog2(16*2)",
            "((1+2)*3)+4",
            "(2*(3+4))-5",
            "(6+7*8)/9",
            "clog2(64)-clog2(8)",
            "1+(2*3)-(4/2)",
            "((1+2)+(3+4))%5",
            "clog2(8)*(2+3)",
            "(3+clog2(16))/2",
            "((2+3)*clog2(4))-1",
            "clog2(32)+(1*2)",
            "1+(clog2(8)*3)",
            "(4+5)*(clog2(16)-2)",
            "(1*2)+(3*4)",
            "clog2(2+6)",
            "(7+8)-(clog2(32)/4)",
            "(1+2+3+4+5)",
        ];

        let test_lambdas: [fn() -> usize; 48] = [
            || 0,
            || 1,
            || 42,
            || 123,
            || clog2(1),
            || clog2(8),
            || 1 + 2,
            || 4 - 3,
            || 5 * 6,
            || 7 / 8,
            || 9 % 2,
            || 1 + 2 + 3,
            || 4 * 5 - 6,
            || 7 + 8 * 9,
            || (1 + 2) * 3,
            || 4 * (5 + 6),
            || clog2(16) + 2,
            || 3 + clog2(8),
            || clog2(4 * 4),
            || (2 + 3) * (4 + 5),
            || 1 + 2 * 3 - 4,
            || clog2(32) - 5,
            || (10 / 2) + 6,
            || (8 % 3) + clog2(8),
            || 1 + 2 + 3 + 4,
            || 5 * 6 * 7,
            || 8 - 3 + 2,
            || (1 + 1) * (2 + 2),
            || clog2(2) + clog2(8),
            || (1 + 2 + 3) * 4,
            || (5 * 6) - (7 + 8),
            || clog2(16 * 2),
            || ((1 + 2) * 3) + 4,
            || (2 * (3 + 4)) - 5,
            || (6 + 7 * 8) / 9,
            || clog2(64) - clog2(8),
            || 1 + (2 * 3) - (4 / 2),
            || ((1 + 2) + (3 + 4)) % 5,
            || clog2(8) * (2 + 3),
            || (3 + clog2(16)) / 2,
            || ((2 + 3) * clog2(4)) - 1,
            || clog2(32) + (1 * 2),
            || 1 + (clog2(8) * 3),
            || (4 + 5) * (clog2(16) - 2),
            || (1 * 2) + (3 * 4),
            || clog2(2 + 6),
            || (7 + 8) - (clog2(32) / 4),
            || 1 + 2 + 3 + 4 + 5,
        ];

        let map: HashMap<String, usize> = HashMap::new();

        for i in 0..test_str.len() {
            let mut offset: usize = 0;

            eprintln!("Iteration: {}, expression: {}", i, test_str[i]);

            let exp: Rc<Box<dyn CompileTimeExpression>> =
                parse_compile_time_expression(test_str[i], &mut offset).unwrap();

            eprintln!("Expression: {:?}", exp);

            assert_eq!(exp.calculate(&map).unwrap(), test_lambdas[i]());
        }
    }

    #[test]
    fn test_parse_parametric_expressions() {
        let test_str: [&str; _] = [
            "x",
            "y",
            "z",
            "x+y",
            "y-z",
            "x*y",
            "z/2",
            "x%y",
            "x+1",
            "y-3",
            "z*4",
            "clog2(x)",
            "clog2(y+1)",
            "clog2(2*z)",
            "x+y+z",
            "x*2+y",
            "z+(x*3)",
            "(x+y)*z",
            "clog2(x)+1",
            "1+clog2(y)",
            "(x+2)*(y+3)",
            "z*(x+y)",
            "(x+clog2(y))*2",
            "(x*y)+(y*z)",
            "clog2(x+y)",
            "(x+1)+(y+2)+(z+3)",
            "(x*y*z)",
            "clog2(x*y)",
            "x+clog2(y*z)",
            "(x+clog2(y))*(z+2)",
            "(x+y+z)/2",
            "(x*2)+(y*3)+(z*4)",
            "clog2(x*2+y)",
            "(x+y)*(z+clog2(4))",
            "x+(y*clog2(8))",
            "clog2(x)+clog2(y)+clog2(z)",
            "(x+y+1)*(z+2)",
            "(x*clog2(y))+z",
            "(x+y)+(clog2(z)+1)",
            "clog2(x+y*z)",
            "(x+clog2(y+z))*2",
            "x+(y+z)+1",
            "(x*y)+(clog2(y)+z)",
            "(x+2)*(y+clog2(z))",
            "clog2(x*y+z)",
            "(x+clog2(y))+(z+1)",
            "(x*y)+clog2(z)",
        ];

        let test_lambdas: [fn(usize, usize, usize) -> usize; _] = [
            |x, _y, _z| x,
            |_x, y, _z| y,
            |_x, _y, z| z,
            |x, y, _z| x + y,
            |_x, y, z| y - z,
            |x, y, _z| x * y,
            |_x, _y, z| z / 2,
            |x, y, _z| x % y,
            |x, _y, _z| x + 1,
            |_x, y, _z| y - 3,
            |_x, _y, z| z * 4,
            |x, _y, _z| clog2(x),
            |_x, y, _z| clog2(y + 1),
            |_x, _y, z| clog2(2 * z),
            |x, y, z| x + y + z,
            |x, y, _z| x * 2 + y,
            |x, _y, z| z + (x * 3),
            |x, y, z| (x + y) * z,
            |x, _y, _z| clog2(x) + 1,
            |_x, y, _z| 1 + clog2(y),
            |x, y, _z| (x + 2) * (y + 3),
            |x, y, z| z * (x + y),
            |x, y, _z| (x + clog2(y)) * 2,
            |x, y, z| (x * y) + (y * z),
            |x, y, _z| clog2(x + y),
            |x, y, z| (x + 1) + (y + 2) + (z + 3),
            |x, y, z| x * y * z,
            |x, y, _z| clog2(x * y),
            |x, y, z| x + clog2(y * z),
            |x, y, z| (x + clog2(y)) * (z + 2),
            |x, y, z| (x + y + z) / 2,
            |x, y, z| (x * 2) + (y * 3) + (z * 4),
            |x, y, _z| clog2(x * 2 + y),
            |x, y, z| (x + y) * (z + clog2(4)),
            |x, y, _z| x + (y * clog2(8)),
            |x, y, z| clog2(x) + clog2(y) + clog2(z),
            |x, y, z| (x + y + 1) * (z + 2),
            |x, y, z| (x * clog2(y)) + z,
            |x, y, z| (x + y) + (clog2(z) + 1),
            |x, y, z| clog2(x + y * z),
            |x, y, z| (x + clog2(y + z)) * 2,
            |x, y, z| x + (y + z) + 1,
            |x, y, z| (x * y) + (clog2(y) + z),
            |x, y, z| (x + 2) * (y + clog2(z)),
            |x, y, z| clog2(x * y + z),
            |x, y, z| (x + clog2(y)) + (z + 1),
            |x, y, z| (x * y) + clog2(z),
        ];

        let mut map: HashMap<String, usize> = HashMap::new();

        map.insert("x".to_string(), 8);
        map.insert("y".to_string(), 8);
        map.insert("z".to_string(), 8);

        let mut rng = rand::rng();

        for _ in 0..200 {
            map.insert("x".to_string(), rng.random_range(0..=200));
            map.insert("y".to_string(), rng.random_range(0..=200));
            map.insert("z".to_string(), rng.random_range(0..=200));

            for i in 0..test_str.len() {
                let mut offset: usize = 0;

                eprintln!("Iteration: {}, expression: {}", i, test_str[i]);

                let exp: Rc<Box<dyn CompileTimeExpression>> =
                    parse_compile_time_expression(test_str[i], &mut offset).unwrap();

                eprintln!("Expression: {:?}", exp);

                let calc: Option<usize> = exp.calculate(&map);

                let res =
                    std::panic::catch_unwind(|| test_lambdas[i](map["x"], map["y"], map["z"]));

                if calc.is_some() || res.is_ok() {
                    assert_eq!(calc.unwrap(), res.unwrap());
                }
            }
        }
    }

    #[test]
    fn test_parser() {
        crate::parser::parse_str(include_str!("../examples/some_module.fr")).unwrap();
    }
}
