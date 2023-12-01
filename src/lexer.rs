use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    True,
    False,
    Assignment,
    OpenRoundParenthesis,
    CloseRoundParenthesis,
    OpenGraphParenthesis,
    CloseGraphParenthesis,
    OpenSquareParenthesis,
    CloseSquareParenthesis,
    Integer(i64),
    Identifier(String),
    String(String),
    // control
    While,
    If,
    Else,
    // logic
    ExclamationPoint,
    LogicalOr, // todo: it's unsupported as I ended up not needing it.
    // Math:
    Addition,
    Multiplication,
    Semicolon,
    Equality,
    Disequality,
    Let,
    LessThan,
    In,
    Print,
}

// should take in input the variables and functions I've seen until now.
pub fn parse(line: &str) -> Result<Vec<Token>> {
    let mut tokens = vec![];
    let mut chars = line.chars().peekable();
    let index = 0;
    while let Some(&c) = chars.peek() {
        let token = match c {
            '0'..='9' => {
                let mut number = 0;
                while let Some(&digit) = chars.peek() {
                    if digit.is_digit(10) {
                        number = number * 10 + digit.to_digit(10).unwrap() as i64;
                        chars.next(); // Move to the next character
                    } else {
                        break;
                    }
                }
                Token::Integer(number)
            }
            '(' => {
                chars.next();
                Token::OpenRoundParenthesis
            }
            ')' => {
                chars.next();
                Token::CloseRoundParenthesis
            }
            '=' => {
                chars.next();
                let next_char = chars.peek();
                match next_char {
                    Some(&'=') => {
                        chars.next();
                        Token::Equality
                    }
                    _ => bail!("Syntax error: expected '=' after '=' on line '{line}'."),
                }
            }
            '|' => {
                chars.next();
                let next_char = chars.peek();
                match next_char {
                    Some(&'|') => {
                        chars.next();
                        Token::LogicalOr
                    }
                    _ => {
                        bail!("Syntax error: expected '|' after '|' on line '{line}'.");
                    }
                }
            }
            '!' => {
                chars.next();
                let next_char = chars.peek();
                match next_char {
                    Some(&'=') => {
                        chars.next();
                        Token::Disequality
                    }
                    _ => {
                        bail!("Syntax error: unexpect char after !, : {next_char:?}")
                    }
                }
            }
            '+' => {
                chars.next();
                Token::Addition
            }
            '*' => {
                chars.next();
                Token::Multiplication
            }
            ';' => {
                chars.next();
                Token::Semicolon
            }
            '<' => {
                chars.next();
                Token::LessThan
            }
            '{' => {
                chars.next();
                Token::OpenGraphParenthesis
            }
            '[' => {
                chars.next();
                Token::OpenSquareParenthesis
            }
            ']' => {
                chars.next();
                Token::CloseSquareParenthesis
            }
            '}' => {
                chars.next();
                Token::CloseGraphParenthesis
            }
            ':' => {
                chars.next();
                let next_char = chars.peek();
                match next_char {
                    Some(&'=') => {
                        chars.next();
                        Token::Assignment
                    }
                    _ => {
                        bail!("Syntax error: expected '=' after ':' on line '{line}'.");
                    }
                }
            }
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
                continue;
            }
            '"' => {
                let mut string = String::new();
                chars.next();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next();
                        break;
                    } else {
                        string.push(ch);
                        chars.next();
                    }
                }
                Token::String(string.replace("\\n", "\n"))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut identifier = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        identifier.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match identifier.as_str() {
                    "while" => Token::While,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "true" => Token::True,
                    "false" => Token::False,
                    "let" => Token::Let,
                    "in" => Token::In,
                    "print" => Token::Print,
                    _ => Token::Identifier(identifier),
                }
            }
            _ => {
                bail!("Error, unrecognized char: {c} on line '{line}'");
            }
        };
        tokens.push(token);
    }
    Ok(tokens)
}

#[cfg(test)]
mod test {
    use crate::lexer::Token::{
        Assignment, CloseGraphParenthesis, Identifier, Let, OpenGraphParenthesis, True,
    };
    use crate::lexer::{parse, Token};
    use std::{assert_eq, matches, println, vec};

    fn expect_single_number(line: &str, expected: Token) {
        let tokens = parse(line).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], expected));
    }
    #[test]
    fn test_number_parsing() {
        expect_single_number("1", Token::Integer(1));
        expect_single_number("123", Token::Integer(123));
        expect_single_number("0", Token::Integer(0));
        expect_single_number("10", Token::Integer(10));
        expect_single_number("100", Token::Integer(100));
        expect_single_number("1000", Token::Integer(1000));
        expect_single_number("10000", Token::Integer(10000));
        expect_single_number("100000", Token::Integer(100000));
        expect_single_number("    1000000   ", Token::Integer(1000000));
    }

    #[test]
    fn test_while() {
        use Token::{Addition, Identifier, Integer, Semicolon, While};
        let program = "while true { let i := 10 + 5; }";
        let tokens = parse(program).unwrap();
        assert_eq!(
            tokens,
            vec![
                While,
                True,
                OpenGraphParenthesis,
                Let,
                Identifier("i".to_string()),
                Assignment,
                Integer(10),
                Addition,
                Integer(5),
                Semicolon,
                CloseGraphParenthesis
            ]
        );
    }

    #[test]
    fn test_keywords_identifiers_parsing() {
        let line = "while ";
    }
    #[test]
    fn test_parse_line() {
        let program = r#"      
let quiz_input := "
1abc2
";
let sum := 0;
let index := 0;
while index < 42 {
    let is_first_digit_found := false;
    let first_digit_found := 0;
    let last_digit_found := 0;
    while quiz_input[index] != "\n" {
        if quiz_input[index] in "0123456789" {
            if is_first_digit_found != false {
                first_digit_found := quiz_input[index];
                is_first_digit_found := true;
            } 
            last_digit_found := quiz_input[index];
        }
        index := index + 1;
    }
    let last_digit_found_mult := last_digit_found * 10;
    sum := sum + last_digit_found_mult;
    sum := sum + first_digit_found;
    index := index + 1;
}
"#;
        let tokens = parse(program).unwrap();
        println!("{:?}", tokens);
        //assert_eq!(tokens, expected);
    }
}
