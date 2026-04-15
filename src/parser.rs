use std::{rc::Rc, vec};

use crate::{
    expression::{
        BinaryExpression, BinaryOperation, CompileTimeExpression, Const, Parameter, UnaryExpression,
    },
    syntax::{Module, ParameterDeclaration, ParameterType, Port, PortDir, PortModifier},
    tokens::{self, Keyword, Token, TokenType},
};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEnd,
    UnexpectedToken(Token),
}

fn check_token(tokens: &[Token], expected: TokenType) -> bool {
    if tokens.len() == 0 {
        false
    } else {
        std::mem::discriminant(&tokens[0].token_type) == std::mem::discriminant(&expected)
    }
}

fn expect_tokens(tokens: &[Token]) -> Result<(), ParseError> {
    if tokens.len() > 0 {
        Ok(())
    } else {
        eprintln!("Unexpected End, line: {}", line!());
        Err(ParseError::UnexpectedEnd)
    }
}

fn parse_token<'a>(tokens: &'a [Token], expected: TokenType) -> Result<&'a [Token], ParseError> {
    expect_tokens(tokens)?;

    if std::mem::discriminant(&tokens[0].token_type) == std::mem::discriminant(&expected) {
        Ok(&tokens[1..])
    } else {
        eprintln!("Unexpected token, line: {}", line!());
        Err(ParseError::UnexpectedToken(tokens[0].clone()))
    }
}

fn parse_name<'a>(tokens: &'a [Token]) -> Result<(&'a [Token], String), ParseError> {
    expect_tokens(tokens)?;

    if let TokenType::Name(name) = &tokens[0].token_type {
        Ok((&tokens[1..], name.clone()))
    } else {
        eprintln!("Unexpected token, line: {}", line!());
        Err(ParseError::UnexpectedToken(tokens[0].clone()))
    }
}

fn parse_number<'a>(tokens: &'a [Token]) -> Result<(&'a [Token], String), ParseError> {
    expect_tokens(tokens)?;

    if let TokenType::Number(name) = &tokens[0].token_type {
        Ok((&tokens[1..], name.clone()))
    } else {
        eprintln!("Unexpected token, line: {}", line!());
        Err(ParseError::UnexpectedToken(tokens[0].clone()))
    }
}

fn parse_keyword<'a>(tokens: &'a [Token]) -> Result<(&'a [Token], Keyword), ParseError> {
    expect_tokens(tokens)?;

    if let TokenType::Keyword(keyword) = &tokens[0].token_type {
        Ok((&tokens[1..], *keyword))
    } else {
        eprintln!("Unexpected token, line: {}", line!());
        Err(ParseError::UnexpectedToken(tokens[0].clone()))
    }
}

fn parse_brackets<'a>(
    tokens: &'a [Token],
    open: TokenType,
    close: TokenType,
) -> Result<(&'a [Token], &'a [Token]), ParseError> {
    expect_tokens(tokens)?;

    if tokens[0].token_type != open {
        eprintln!("Unexpected token, line: {}", line!());
        return Err(ParseError::UnexpectedToken(tokens[0].clone()));
    }

    let mut count: i32 = 1;
    let mut i: usize = 1;

    while count > 0 && i < tokens.len() {
        if tokens[i].token_type == open {
            count += 1;
        } else if tokens[i].token_type == close {
            count -= 1;
        }
        i += 1;
    }

    if count == 0 {
        Ok((&tokens[i..], &tokens[1..(i - 1)]))
    } else {
        eprintln!("Unexpected End, line: {}, tokens: {:?}", line!(), tokens);
        Err(ParseError::UnexpectedEnd)
    }
}

fn parse_port_list<'a>(tokens: &'a [Token]) -> Result<(&'a [Token], Option<Port>), ParseError> {
    if tokens.len() == 0 {
        return Ok((tokens, None));
    }

    let (tokens, name) = parse_name(tokens)?;
    let tokens = parse_token(tokens, TokenType::Colon)?;

    let (tokens, direction) = match parse_keyword(tokens)? {
        (tokens, Keyword::Input) => (tokens, PortDir::Input),
        (tokens, Keyword::Output) => (tokens, PortDir::Output),
        (tokens, Keyword::Inout) => (tokens, PortDir::Inout),
        _ => {
            eprintln!("Unexpected token, line: {}", line!());
            return Err(ParseError::UnexpectedToken(tokens[0].clone()));
        }
    };

    let (width, tokens): (Rc<Box<dyn CompileTimeExpression>>, &[Token]) =
        if check_token(tokens, TokenType::OpenBracket) {
            let (tokens, expression) =
                parse_brackets(tokens, TokenType::OpenBracket, TokenType::CloseBracket)?;

            let (remained_tokens, expression) = parse_compile_time_expression(expression)?;

            if remained_tokens.len() != 0 {
                return Err(ParseError::UnexpectedToken(remained_tokens[0].clone()));
            }

            (expression, tokens)
        } else {
            (Rc::new(Box::new(Const { value: 1 })), tokens)
        };

    let (modifiers, tokens): (Vec<PortModifier>, &[Token]) = if check_token(tokens, TokenType::Less)
    {
        let (tokens, _expression) = parse_brackets(tokens, TokenType::Less, TokenType::More)?;
        // TODO Modifiers parser
        (vec![], tokens)
    } else {
        (vec![], tokens)
    };

    Ok((
        tokens,
        Some(Port {
            name,
            direction,
            width,
            modifiers,
        }),
    ))
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

pub fn parse_compile_time_expression<'a>(
    tokens: &'a [Token],
) -> Result<(&'a [Token], Rc<Box<dyn CompileTimeExpression>>), ParseError> {
    expect_tokens(tokens)?;

    let mut expressions: Vec<Rc<Box<dyn CompileTimeExpression>>> = vec![];
    let mut operations: Vec<BinaryOperation> = vec![];

    let mut i: usize = 0;

    while i < tokens.len() {
        match &tokens[i].token_type {
            TokenType::OpenParenthesis => {
                let (_rest_tokens, inner_tokens) = parse_brackets(
                    &tokens[i..],
                    TokenType::OpenParenthesis,
                    TokenType::CloseParenthesis,
                )?;

                let (remain_tokens, exp) = parse_compile_time_expression(inner_tokens)?;

                if remain_tokens.len() != 0 {
                    return Err(ParseError::UnexpectedToken(remain_tokens[0].clone()));
                }

                expressions.push(exp);
                i += inner_tokens.len() + 2;
            }

            TokenType::Plus => {
                operations.push(BinaryOperation::Add);
                i += 1;
            }
            TokenType::Minus => {
                operations.push(BinaryOperation::Sub);
                i += 1;
            }
            TokenType::Multiply => {
                operations.push(BinaryOperation::Mul);
                i += 1;
            }
            TokenType::Divide => {
                operations.push(BinaryOperation::Div);
                i += 1;
            }
            TokenType::Mod => {
                operations.push(BinaryOperation::Mod);
                i += 1;
            }
            TokenType::Keyword(Keyword::Clog2) => {
                let (_rest_tokens, inner_tokens) = parse_brackets(
                    &tokens[(i + 1)..],
                    TokenType::OpenParenthesis,
                    TokenType::CloseParenthesis,
                )?;

                let (remain_tokens, exp) = parse_compile_time_expression(inner_tokens)?;

                if remain_tokens.len() != 0 {
                    return Err(ParseError::UnexpectedToken(remain_tokens[0].clone()));
                }

                expressions.push(Rc::new(Box::new(UnaryExpression {
                    exp,
                    op: crate::expression::UnaryOperation::Clog2,
                })));
                i += inner_tokens.len() + 3;
            }
            TokenType::Keyword(Keyword::Max) => {
                let (_rest_tokens, inner_tokens) = parse_brackets(
                    &tokens[(i + 1)..],
                    TokenType::OpenParenthesis,
                    TokenType::CloseParenthesis,
                )?;

                let (remain_tokens, lhs) = parse_compile_time_expression(inner_tokens)?;
                let (remain_tokens, rhs) = parse_compile_time_expression(&remain_tokens[1..])?;

                if remain_tokens.len() != 0 {
                    return Err(ParseError::UnexpectedToken(remain_tokens[0].clone()));
                }

                expressions.push(Rc::new(Box::new(BinaryExpression {
                    lhs,
                    rhs,
                    op: crate::expression::BinaryOperation::Max,
                })));
                i += inner_tokens.len() + 3;
            }
            TokenType::Keyword(Keyword::Min) => {
                let (_rest_tokens, inner_tokens) = parse_brackets(
                    &tokens[(i + 1)..],
                    TokenType::OpenParenthesis,
                    TokenType::CloseParenthesis,
                )?;

                let (remain_tokens, lhs) = parse_compile_time_expression(inner_tokens)?;
                let (remain_tokens, rhs) = parse_compile_time_expression(&remain_tokens[1..])?;

                if remain_tokens.len() != 0 {
                    return Err(ParseError::UnexpectedToken(remain_tokens[0].clone()));
                }

                expressions.push(Rc::new(Box::new(BinaryExpression {
                    lhs,
                    rhs,
                    op: crate::expression::BinaryOperation::Min,
                })));
                i += inner_tokens.len() + 3;
            }
            TokenType::Number(number) => {
                expressions.push(Rc::new(Box::new(Const {
                    value: number.parse().unwrap(),
                })));
                i += 1;
            }
            TokenType::Name(name) => {
                expressions.push(Rc::new(Box::new(Parameter { name: name.clone() })));
                i += 1;
            }
            TokenType::Comma => {
                break;
            }
            _ => {
                eprintln!("Unexpected token, line: {}", line!());
                return Err(ParseError::UnexpectedToken(tokens[i].clone()));
            }
        }
    }

    Ok((
        &tokens[i..],
        collapse_expression(&mut expressions, &mut operations),
    ))
}

fn split_tokens<'a>(tokens: &'a [Token], separator: TokenType) -> Vec<&'a [Token]> {
    let mut res: Vec<&[Token]> = Vec::new();

    let mut start: usize = 0;

    for (i, token) in tokens.iter().enumerate() {
        if token.token_type == separator {
            res.push(&tokens[start..i]);
            start = i + 1;
        }
    }

    if start < tokens.len() {
        res.push(&tokens[start..]);
    }

    res
}

fn parse_module<'a>(tokens: &'a [Token]) -> Result<(&'a [Token], Module), ParseError> {
    let (tokens, name) = parse_name(tokens)?;

    let (tokens, parameters) = if check_token(tokens, TokenType::Less) {       
        let (tokens, parameters_tokens) = parse_brackets(tokens, TokenType::Less, TokenType::More)?;
        let parameters_tokens: Vec<&[Token]> = split_tokens(parameters_tokens, TokenType::Comma);
        let mut parameters: Vec<ParameterDeclaration> = Vec::new();

        for tokens in parameters_tokens {
            let (tokens, name) = parse_name(tokens)?;
            let tokens = parse_token(tokens, TokenType::Colon)?;

            // TODO support other parameters type
            let parameter_type = if tokens.first().map(|token| &token.token_type)
                == Some(&TokenType::Keyword(Keyword::Unsigned))
            {
                ParameterType::Unsigned
            } else {
                if let Some(token) = tokens.first() {
                    return Err(ParseError::UnexpectedToken(token.clone()));
                } else {
                    return Err(ParseError::UnexpectedEnd);
                }
            };

            let tokens = &tokens[1..];

            let default: Option<usize> = if check_token(tokens, TokenType::Less) {
                let (remaining_tokens, tokens) =
                    parse_brackets(tokens, TokenType::Less, TokenType::More)?;
                if remaining_tokens.len() != 0 {
                    return Err(ParseError::UnexpectedToken(tokens[0].clone()));
                }

                // TODO proper universal modifier parser

                expect_tokens(tokens)?;

                let first_token = tokens[0].clone();
                let (tokens, modifier_type) = parse_name(tokens)?;

                if modifier_type.as_str() != "default" {
                    return Err(ParseError::UnexpectedToken(first_token));
                }
                let tokens = parse_token(tokens, TokenType::Colon)?;
                let (tokens, default_value) = parse_number(tokens)?;
                
                if tokens.len() != 0 {
                    return Err(ParseError::UnexpectedToken(tokens[0].clone()));
                }
                eprintln!("ASDASDafbsdfkdsgfhjsdgf");

                Some(str::parse::<usize>(default_value.as_str()).unwrap())
            } else {
                None
            };

            parameters.push(ParameterDeclaration {
                name,
                default,
                parameter_type,
            });
        }

        (tokens, parameters)
    } else {
        (tokens, Vec::new())
    };

    let (tokens, interface) = parse_brackets(
        tokens,
        TokenType::OpenParenthesis,
        TokenType::CloseParenthesis,
    )?;

    let mut interface = interface;

    let mut ports: Vec<Port> = Vec::new();

    loop {
        let port: Option<Port>;

        (interface, port) = parse_port_list(interface)?;

        if let Some(port) = port {
            ports.push(port);
        } else {
            break;
        }

        if !check_token(interface, TokenType::Comma) {
            break;
        } else {
            interface = parse_token(interface, TokenType::Comma)?
        }
    }

    if interface.len() > 0 {
        eprintln!("Unexpected token, line: {}", line!());
        return Err(ParseError::UnexpectedToken(interface[0].clone()));
    }

    let (tokens, _body) = parse_brackets(tokens, TokenType::OpenBrace, TokenType::CloseBrace)?;

    Ok((
        tokens,
        Module {
            name,
            parameters,
            logic: (),
            interface: (),
        },
    ))
}

fn tokenize(string: &str) -> Vec<Token> {
    let tokens: Vec<Token> = crate::tokens::Token::from_str(string)
        .iter()
        .cloned()
        .filter(|token| !token.token_type.is_comment())
        .collect();
    tokens
}

pub fn parse_str(string: &str) -> Result<(), ParseError> {
    let tokens: Vec<Token> = tokenize(string);
    let mut tokens: &[Token] = tokens.as_slice();

    while tokens.len() > 0 {
        let token: &Token = &tokens[0];
        match (&token.token_type, token.offset) {
            (TokenType::Keyword(crate::tokens::Keyword::Module), _) => {
                let module: Module;

                (tokens, module) = parse_module(&tokens[1..])?;

                eprintln!("Parsed module {}", module.name);
                eprintln!("Parameters {:?}", module.parameters);
            }
            _ => {
                eprintln!("Unexpected token {:?}", token);
                return Err(ParseError::UnexpectedToken(token.clone()));
            }
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
            " ",
            " 1",
            " x",
            " clog2(8)",
            " 1+2",
            " x+y",
            " (1+2)",
            " (x+y)",
            " 1+2*3",
            " (1+2)*3",
            " 1+(2*3)",
            " ((1))",
            " ((x))",
            " clog2((8))",
            " x+(y*z)",
            " (x+1)+(y+2)",
            " (a+b)*(c+d)",
            " (1+clog2(8))*2",
            " (x*y)+(z/3)",
            " (a+b)+((c+d)*e)",
            " 1+2+(3+4)",
            " (x+y)+(z+1)",
            " clog2(x+y)",
            " (1+2)*(3+4)",
            " ((1+2)*3)+4",
            " x+((y+z)*2)",
            " (clog2(x)+clog2(y))*z",
            " (1+2)*(x+3)",
            " (a*b)+(c*d)",
            " ((x+y)+z)",
            " ((1+2)+(3+4))",
            " (x+(y+z))*2",
            " ((x))",
            " x+(y*clog2(8))",
            " (1+clog2(4))+(x*2)",
            " ((a*b)+c)",
            " (x+y+z)",
            " ((1+2)*(3+4))",
            " x+((y+z)+clog2(8))",
            " (clog2(2*x)+y)-z",
            " (x*y)+(clog2(y+z))",
            " (x+(y*z))+((a+b)*c)",
            " ((x*y)+(z/2))",
            " (x*2)+((y+3)*z)",
            " ((x+1)+y)+z",
            " (1+(2*(3+4)))",
            " (x+((y+z)+1))",
            " (x+(y+(z*2)))",
        ];

        for i in 0..test_str.len() {
            eprintln!("Iteration: {}", i);

            assert_eq!(
                parse_brackets(
                    &tokenize(test_str[i]),
                    TokenType::OpenParenthesis,
                    TokenType::CloseParenthesis
                )
                .unwrap()
                .1,
                tokenize(ref_str[i]).as_slice(),
            );
        }
    }

    #[test]
    fn test_parse_expressions() {
        let test_str: [&str; 49] = [
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
            "min(max(2, 4), max(3, 8))",
        ];

        let test_lambdas: [fn() -> usize; 49] = [
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
            || 4,
        ];

        let map: HashMap<String, usize> = HashMap::new();

        for i in 0..test_str.len() {
            eprintln!("Iteration: {}, expression: {}", i, test_str[i]);

            let tokens = tokenize(test_str[i]);
            let (tokens, exp) = parse_compile_time_expression(&tokens).unwrap();

            assert_eq!(tokens.len(), 0);
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

        for i in 0..test_str.len() {
            eprintln!("Iteration: {}, expression: {}", i, test_str[i]);

            let tokens = tokenize(test_str[i]);
            let (tokens, exp) = parse_compile_time_expression(&tokens).unwrap();

            assert_eq!(tokens.len(), 0);
            eprintln!("Expression: {:?}", exp);

            for _ in 0..200 {
                map.insert("x".to_string(), rng.random_range(0..=200));
                map.insert("y".to_string(), rng.random_range(0..=200));
                map.insert("z".to_string(), rng.random_range(0..=200));

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
    fn test_split_tokens() {
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

        for i in 0..test_str.len() {
            eprintln!("Iteration: {}, expression: {}", i, test_str[i]);

            let tokens = tokenize(test_str[i]);
            let (tokens, exp) = parse_compile_time_expression(&tokens).unwrap();

            assert_eq!(tokens.len(), 0);
            eprintln!("Expression: {:?}", exp);

            for _ in 0..200 {
                map.insert("x".to_string(), rng.random_range(0..=200));
                map.insert("y".to_string(), rng.random_range(0..=200));
                map.insert("z".to_string(), rng.random_range(0..=200));

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
