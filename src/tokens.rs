#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Keyword {
    Module,
    Function,
    If,
    Else,
    Case,
    Always,
    Input,
    Output,
    Inout,
    Min,
    Max,
    Clog2,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    Keyword(Keyword),
    Name(String),
    Number(String),
    LineComment(String),
    BlockComment(String),
    Comma,
    Colon,
    Semicolon,
    OpenParenthesis,
    CloseParenthesis,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Less,
    LessEq,
    More,
    MoreEq,
    Equal,
    Assign,
    Plus,
    Minus,
    Multiply,
    Divide,
    Mod,
    BoolOr,
    Or,
    BoolAnd,
    And,
    BoolXor,
    Xor,
    BoolNot,
    Not,

    Unknown,
}

impl TokenType {
    pub fn is_comment(&self) -> bool {
        match self {
            Self::LineComment(_) => true,
            Self::BlockComment(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub offset: usize,
    pub token_type: TokenType,
}

impl Token {
    const TOKEN_MAP: &[(&str, TokenType)] = &[
        (",", TokenType::Comma),
        (":", TokenType::Colon),
        (";", TokenType::Semicolon),
        ("(", TokenType::OpenParenthesis),
        (")", TokenType::CloseParenthesis),
        ("[", TokenType::OpenBracket),
        ("]", TokenType::CloseBracket),
        ("{", TokenType::OpenBrace),
        ("}", TokenType::CloseBrace),
        ("<=", TokenType::LessEq),
        ("<", TokenType::Less),
        (">=", TokenType::MoreEq),
        (">", TokenType::More),
        ("==", TokenType::Equal),
        ("=", TokenType::Assign),
        ("+", TokenType::Plus),
        ("-", TokenType::Minus),
        ("*", TokenType::Multiply),
        ("/", TokenType::Divide),
        ("%", TokenType::Mod),
        ("||", TokenType::BoolOr),
        ("|", TokenType::Or),
        ("&&", TokenType::BoolAnd),
        ("&", TokenType::And),
        ("^^", TokenType::BoolXor),
        ("^", TokenType::Xor),
        ("!", TokenType::BoolNot),
        ("~", TokenType::Not),
        ("module", TokenType::Keyword(Keyword::Module)),
        ("function", TokenType::Keyword(Keyword::Function)),
        ("if", TokenType::Keyword(Keyword::If)),
        ("else", TokenType::Keyword(Keyword::Else)),
        ("case", TokenType::Keyword(Keyword::Case)),
        ("always", TokenType::Keyword(Keyword::Always)),
        ("input", TokenType::Keyword(Keyword::Input)),
        ("output", TokenType::Keyword(Keyword::Output)),
        ("inout", TokenType::Keyword(Keyword::Inout)),
        // ("static::min", TokenType::Keyword(Keyword::Min)), // TODO?
        // ("static::max", TokenType::Keyword(Keyword::Max)),
        // ("static::clog2", TokenType::Keyword(Keyword::Clog2)),
        ("min", TokenType::Keyword(Keyword::Min)),
        ("max", TokenType::Keyword(Keyword::Max)),
        ("clog2", TokenType::Keyword(Keyword::Clog2)),
    ];

    fn is_au(symbol: char) -> bool {
        symbol.is_ascii_alphabetic() || symbol == '_'
    }

    fn is_aun(symbol: char) -> bool {
        symbol.is_alphanumeric() || symbol == '_'
    }

    pub fn from_str(string: &str) -> Vec<Self> {
        let mut offset: usize = 0;
        let mut tokens: Vec<Self> = Vec::new();

        'main_loop: while offset < string.len() {
            if string[offset..].starts_with("//") {
                let mut i: usize = 2;
                while i + offset < string.len() && !string[(offset + i)..].starts_with("\n") {
                    i += 1;
                }
                tokens.push(Self {
                    offset,
                    token_type: TokenType::LineComment(
                        string[(offset + 2)..(offset + i)].to_string(),
                    ),
                });
                offset += i;
                continue 'main_loop;
            }

            if string[offset..].starts_with("/*") {
                let mut i: usize = 2;
                while i + offset < string.len() && !string[(offset + i)..].starts_with("*/") {
                    i += 1;
                }
                tokens.push(Self {
                    offset,
                    token_type: TokenType::BlockComment(
                        string[(offset + 2)..(offset + i)].to_string(),
                    ),
                });

                offset += i + 2;
                continue 'main_loop;
            }

            for (token_str, token_type) in Self::TOKEN_MAP {
                if string[offset..].starts_with(token_str) {
                    let token_type: TokenType = token_type.clone();

                    let end: usize = offset + token_str.len();
                    if let TokenType::Keyword(_) = token_type
                        && end < string.len()
                    {
                        let next: char = string.as_bytes()[end] as char;
                        if Self::is_aun(next) {
                            continue;
                        }
                    }

                    tokens.push(Self { offset, token_type });

                    offset += token_str.len();
                    continue 'main_loop;
                }
            }

            if Self::is_au(string[offset..].as_bytes()[0].into()) {
                let mut i: usize = 1;

                while offset + i < string.len()
                    && Self::is_aun(string[(offset + i)..].as_bytes()[0].into())
                {
                    i += 1;
                }

                tokens.push(Self {
                    offset,
                    token_type: TokenType::Name(string[offset..(offset + i)].into()),
                });
                offset += i;
                continue 'main_loop;
            }
            if (string[offset..].as_bytes()[0] as char).is_ascii_digit() {
                let mut i: usize = 1;

                while offset + i < string.len()
                    && Self::is_aun(string[(offset + i)..].as_bytes()[0].into())
                {
                    i += 1;
                }

                tokens.push(Self {
                    offset,
                    token_type: TokenType::Number(string[offset..(offset + i)].into()),
                });
                offset += i;
                continue 'main_loop;
            }

            if !(string[offset..].as_bytes()[0] as char).is_ascii_whitespace() {
                tokens.push(Self {
                    offset,
                    token_type: TokenType::Unknown,
                });
            }

            offset += 1;
        }

        tokens
    }
}

//// Tests

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tokens(string: &str, tokens: &[TokenType]) {
        {
            {
                let parsed_tokens: Vec<Token> = Token::from_str(string);

                eprintln!("parsed tokens: {:?}", parsed_tokens);
                eprintln!("expected tokens: {:?}", tokens);

                assert_eq!(
                    tokens.len(),
                    parsed_tokens.len(),
                    "token count mismatch\ninput: {:?}",
                    string,
                );

                // for (i, (tok, exp)) in kinds.iter().zip(expected.iter()).enumerate() {
                for i in 0..tokens.len() {
                    assert_eq!(
                        tokens[i], parsed_tokens[i].token_type,
                        "token {} mismatch\nexpected: {:?}\nactual: {:?}",
                        i, tokens[i], parsed_tokens[i]
                    );
                }
            }
        };
    }
    #[test]
    fn keyword_basic() {
        assert_tokens(
            "module function if else case always",
            &[
                TokenType::Keyword(Keyword::Module),
                TokenType::Keyword(Keyword::Function),
                TokenType::Keyword(Keyword::If),
                TokenType::Keyword(Keyword::Else),
                TokenType::Keyword(Keyword::Case),
                TokenType::Keyword(Keyword::Always),
            ],
        );
    }
    #[test]
    fn keyword_boundary() {
        assert_tokens(
            "moduleX ifx else1",
            &[
                TokenType::Name("moduleX".to_string()),
                TokenType::Name("ifx".to_string()),
                TokenType::Name("else1".to_string()),
            ],
        );
    }

    #[test]
    fn identifiers() {
        assert_tokens(
            "_a a1 a_b_c __foo123",
            &[
                TokenType::Name("_a".to_string()),
                TokenType::Name("a1".to_string()),
                TokenType::Name("a_b_c".to_string()),
                TokenType::Name("__foo123".to_string()),
            ],
        );
    }

    #[test]
    fn numbers() {
        assert_tokens(
            "0 123 10b101001 16hFF 32d255",
            &[
                TokenType::Number("0".to_string()),
                TokenType::Number("123".to_string()),
                TokenType::Number("10b101001".to_string()),
                TokenType::Number("16hFF".to_string()),
                TokenType::Number("32d255".to_string()),
            ],
        );
    }

    #[test]
    fn operators() {
        assert_tokens(
            "< <= > >= == = && & || | ^^ ^ ! ~",
            &[
                TokenType::Less,
                TokenType::LessEq,
                TokenType::More,
                TokenType::MoreEq,
                TokenType::Equal,
                TokenType::Assign,
                TokenType::BoolAnd,
                TokenType::And,
                TokenType::BoolOr,
                TokenType::Or,
                TokenType::BoolXor,
                TokenType::Xor,
                TokenType::BoolNot,
                TokenType::Not,
            ],
        );
    }

    #[test]
    fn mixed_statement() {
        assert_tokens(
            "module foo(input a, input b); // TEST COMMENT",
            &[
                TokenType::Keyword(Keyword::Module),
                TokenType::Name("foo".to_string()),
                TokenType::OpenParenthesis,
                TokenType::Keyword(Keyword::Input),
                TokenType::Name("a".to_string()),
                TokenType::Comma,
                TokenType::Keyword(Keyword::Input),
                TokenType::Name("b".to_string()),
                TokenType::CloseParenthesis,
                TokenType::Semicolon,
                TokenType::LineComment(" TEST COMMENT".to_string()),
            ],
        );
    }

    #[test]
    fn block_comment() {
        assert_tokens(
            "module foo(input a, input b); /*  THIS IS TEST COMMENT
 TEST BLOCK COMMENT
ABOBA AMOGUS*/ input, output",
            &[
                TokenType::Keyword(Keyword::Module),
                TokenType::Name("foo".to_string()),
                TokenType::OpenParenthesis,
                TokenType::Keyword(Keyword::Input),
                TokenType::Name("a".to_string()),
                TokenType::Comma,
                TokenType::Keyword(Keyword::Input),
                TokenType::Name("b".to_string()),
                TokenType::CloseParenthesis,
                TokenType::Semicolon,
                TokenType::BlockComment(
                    "  THIS IS TEST COMMENT
 TEST BLOCK COMMENT
ABOBA AMOGUS"
                        .to_string(),
                ),
                TokenType::Keyword(Keyword::Input),
                TokenType::Comma,
                TokenType::Keyword(Keyword::Output),
            ],
        );
    }

    #[test]
    fn whitespace() {
        assert_tokens(
            " \t\n module \n\t foo ",
            &[
                TokenType::Keyword(Keyword::Module),
                TokenType::Name("foo".to_string()),
            ],
        );
    }

    #[test]
    fn unknown_tokens() {
        let tokens = Token::from_str("@#$");
        assert_eq!(tokens.len(), 3);

        for t in tokens {
            matches!(t.token_type, TokenType::Unknown);
        }
    }

    #[test]
    fn offsets_are_correct() {
        let tokens = Token::from_str("module foo");
        let offsets: Vec<usize> = tokens.iter().map(|t| t.offset).collect();

        assert_eq!(offsets, vec![0, 7]);
    }
}
