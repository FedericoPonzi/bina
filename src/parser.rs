use crate::lexer::Token;
use anyhow::{bail, Result};
use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Integer(i64),
    String(String),
    Boolean(bool),
    Variable(String),
    VariableIndexed(String, Box<Expr>),
}
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    //TODO: these can be deduplicated with a binaryop
    Add(Box<Term>, Box<Term>),
    Multiply(Box<Term>, Box<Term>),
    LogicalOr(Box<Term>, Box<Term>),
    Equality(Box<Term>, Box<Term>),
    DisEquality(Box<Term>, Box<Term>),
    LessThan(Box<Term>, Box<Term>),
    ContainedIn(Box<Term>, Box<Term>),
    TermWrapper(Term),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    If(Box<Expr>, Box<Statement>),    // predicate, block
    While(Box<Expr>, Box<Statement>), // predicate, block
    Block(Vec<Statement>),
    Assignment(String, Box<Expr>, bool), // bool = prefixed by let or not
    Print(Box<Expr>),
}
fn parse_block(input: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Statement> {
    let left_par = input.next();
    if left_par != Some(Token::OpenGraphParenthesis) {
        return Err(anyhow::anyhow!("Expected '{{', received: {left_par:?}."));
    }
    let mut ret = vec![];
    while input.peek() != Some(&Token::CloseGraphParenthesis) {
        let statement = parse_statement(input)?;
        ret.push(statement);
    }
    let _right_par = input.next();
    Ok(Statement::Block(ret))
}
fn parse_while(input: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Statement> {
    let condition = parse_expr(input)?;
    let block = parse_block(input)?;
    Ok(Statement::While(Box::new(condition), Box::new(block)))
}
fn expect_semicolon(t: Option<Token>) -> Result<()> {
    if t != Some(Token::Semicolon) {
        bail!("Expected ';', received: {:?}", t);
    }
    Ok(())
}
fn parse_statement(input: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Statement> {
    match input.next() {
        Some(Token::While) => {
            return parse_while(input);
        }

        Some(Token::If) => {
            let condition = parse_expr(input)?;
            let block = parse_block(input)?;
            Ok(Statement::If(Box::new(condition), Box::new(block)))
        }
        // must be an assignment.
        Some(Token::Identifier(s)) => {
            let identifier = s.to_string();
            let assignment = input.next();
            if assignment != Some(Token::Assignment) {
                bail!("Expected ':=', received: {:?}", assignment);
            }
            let expr = parse_expr(input)?;
            let semicolon = input.next();
            expect_semicolon(semicolon)?;
            Ok(Statement::Assignment(identifier, Box::new(expr), false))
        }
        Some(Token::Let) => {
            let identifier = input.next();
            if let Some(Token::Identifier(identifier)) = identifier {
                println!("Identifier: {:?}", identifier);
                let assignment = input.next();
                if assignment != Some(Token::Assignment) {
                    bail!("Expected ':=', received: {:?}", assignment);
                }
                let expr = parse_expr(input)?;
                let semicolon = input.next();
                expect_semicolon(semicolon)?;
                Ok(Statement::Assignment(identifier, Box::new(expr), true))
            } else {
                bail!("Expected identifier, received: {:?}", identifier);
            }
        }
        Some(Token::Print) => {
            let expr = parse_expr(input)?;
            let semicolon = input.next();
            expect_semicolon(semicolon)?;
            Ok(Statement::Print(Box::new(expr)))
        }
        token => {
            bail!("parse_statement: Unexpected token {:?}", token);
        }
    }
}
fn parse_term(input: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Term> {
    Ok(match input.next() {
        Some(Token::Integer(i)) => Term::Integer(i),
        Some(Token::String(s)) => Term::String(s.to_string()),
        Some(Token::True) => Term::Boolean(true),
        Some(Token::False) => Term::Boolean(false),
        Some(Token::Identifier(s)) => {
            if input.peek() == Some(&Token::OpenSquareParenthesis) {
                let _open = input.next().unwrap();
                let index = parse_expr(input)?;
                let _close = input.next().unwrap();
                Term::VariableIndexed(s.to_string(), Box::new(index))
            } else {
                Term::Variable(s.to_string())
            }
        }
        Some(token) => {
            bail!("parse_term: Unexpected token {:?}", token);
        }
        None => {
            bail!("parse_term: Unexpected end of input");
        }
    })
}
fn parse_expr(input: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Expr> {
    let left = parse_term(input)?;
    let op = input.peek().cloned();
    let ret = match op {
        Some(Token::Multiplication) => {
            let _mult = input.next().unwrap();
            let right = parse_term(input)?;
            Expr::Multiply(Box::new(left), Box::new(right))
        }
        Some(Token::Addition) => {
            let _add = input.next().unwrap();
            let right = parse_term(input)?;
            Expr::Add(Box::new(left), Box::new(right))
        }
        Some(Token::Disequality) => {
            let _disequality = input.next().unwrap();
            let right = parse_term(input)?;
            Expr::DisEquality(Box::new(left), Box::new(right))
        }
        Some(Token::Equality) => {
            let _equality = input.next().unwrap();
            let right = parse_term(input)?;
            Expr::Equality(Box::new(left), Box::new(right))
        }
        Some(Token::LessThan) => {
            let _lt = input.next().unwrap();
            let right = parse_term(input)?;
            Expr::LessThan(Box::new(left), Box::new(right))
        }
        Some(Token::In) => {
            let _in = input.next().unwrap();
            let right = parse_term(input)?;
            Expr::ContainedIn(Box::new(left), Box::new(right))
        }
        Some(other) => Expr::TermWrapper(left),
        _ => {
            bail!("parse_expr: Unexpected token {:?}", op);
        }
    };
    Ok(ret)
}

pub fn parse_input(mut input: Vec<Token>) -> Result<Vec<Statement>> {
    let mut ret = vec![];
    let mut input = input.into_iter().peekable();
    while input.peek().is_some() {
        println!("{:?}", input.peek());
        ret.push(parse_statement(&mut input)?);
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use crate::lexer::Token;
    use crate::lexer::Token::*;
    use crate::parser::{parse_input, Expr, Statement, Term};
    use std::{println, vec};

    #[test]
    fn test_assignment() {
        let input = vec![
            Identifier("x".to_string()),
            Token::Assignment,
            Token::Integer(10),
        ];
        let ret = parse_input(input).unwrap();
        assert_eq!(
            ret,
            vec![Statement::Assignment(
                "x".to_string(),
                Box::new(Expr::TermWrapper(Term::Integer(10))),
                false
            )]
        );
        let input = vec![
            Token::Let,
            Identifier("x".to_string()),
            Token::Assignment,
            Token::Integer(10),
        ];
        let ret = parse_input(input).unwrap();
        assert_eq!(
            ret,
            vec![Statement::Assignment(
                "x".to_string(),
                Box::new(Expr::TermWrapper(Term::Integer(10))),
                true
            )]
        );
        println!("ret: {:?}", ret);
    }
    #[test]
    fn test_parser() {
        let input = vec![
            Token::While,
            True,
            OpenGraphParenthesis,
            Let,
            Identifier("i".to_string()),
            Token::Assignment,
            Token::Integer(10),
            Token::Addition,
            Token::Integer(5),
            Token::Semicolon,
            CloseGraphParenthesis,
        ];
        let ret = parse_input(input);
        println!("ret: {:?}", ret);
    }

    #[test]
    fn test_program() {
        let tokens = vec![
            Let,
            Identifier("quiz_input".to_string()),
            Assignment,
            String("\n1abc2\n".to_string()),
            Semicolon,
            Let,
            Identifier("sum".to_string()),
            Assignment,
            Integer(0),
            Semicolon,
            Let,
            Identifier("index".to_string()),
            Assignment,
            Integer(0),
            Semicolon,
            While,
            Identifier("index".to_string()),
            LessThan,
            Integer(42),
            OpenGraphParenthesis,
            Let,
            Identifier("is_first_digit_found".to_string()),
            Assignment,
            False,
            Semicolon,
            Let,
            Identifier("first_digit_found".to_string()),
            Assignment,
            Integer(0),
            Semicolon,
            Let,
            Identifier("last_digit_found".to_string()),
            Assignment,
            Integer(0),
            Semicolon,
            While,
            Identifier("quiz_input".to_string()),
            OpenSquareParenthesis,
            Identifier("index".to_string()),
            CloseSquareParenthesis,
            Disequality,
            String("\\n".to_string()),
            OpenGraphParenthesis,
            If,
            Identifier("quiz_input".to_string()),
            OpenSquareParenthesis,
            Identifier("index".to_string()),
            CloseSquareParenthesis,
            In,
            String("0123456789".to_string()),
            OpenGraphParenthesis,
            If,
            Identifier("is_first_digit_found".to_string()),
            Disequality,
            False,
            OpenGraphParenthesis,
            Identifier("first_digit_found".to_string()),
            Assignment,
            Identifier("quiz_input".to_string()),
            OpenSquareParenthesis,
            Identifier("index".to_string()),
            CloseSquareParenthesis,
            Semicolon,
            Identifier("is_first_digit_found".to_string()),
            Assignment,
            True,
            Semicolon,
            CloseGraphParenthesis,
            Identifier("last_digit_found".to_string()),
            Assignment,
            Identifier("quiz_input".to_string()),
            OpenSquareParenthesis,
            Identifier("index".to_string()),
            CloseSquareParenthesis,
            Semicolon,
            CloseGraphParenthesis,
            Identifier("index".to_string()),
            Assignment,
            Identifier("index".to_string()),
            Addition,
            Integer(1),
            Semicolon,
            CloseGraphParenthesis,
            Let,
            Identifier("last_digit_found_mult".to_string()),
            Assignment,
            Identifier("last_digit_found".to_string()),
            Multiplication,
            Integer(10),
            Semicolon,
            Identifier("sum".to_string()),
            Assignment,
            Identifier("sum".to_string()),
            Addition,
            Identifier("last_digit_found_mult".to_string()),
            Semicolon,
            Identifier("sum".to_string()),
            Assignment,
            Identifier("sum".to_string()),
            Addition,
            Identifier("first_digit_found".to_string()),
            Semicolon,
            Identifier("index".to_string()),
            Assignment,
            Identifier("index".to_string()),
            Addition,
            Integer(1),
            Semicolon,
            CloseGraphParenthesis,
        ];

        let parse = parse_input(tokens).unwrap();
        dbg!(parse);
    }
}
