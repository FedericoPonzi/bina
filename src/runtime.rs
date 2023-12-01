use crate::parser::Expr::{Add, ContainedIn, DisEquality, Multiply, TermWrapper};
use crate::parser::{Expr, Statement, Term};
use anyhow::{anyhow, bail, Context, Result};
use log::debug;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Number(i64),
    Boolean(bool),
    String(String),
}
type Environment = HashMap<String, Value>;

// todo: right now, all variables are basically global
type EnvironmentStack = Vec<Environment>;

fn evaluate_assignment(
    mut env: Environment,
    variable_name: String,
    expr: Box<Expr>,
    is_let: bool,
) -> Result<Environment> {
    let value = eval_expr(&env, expr)?;
    env.insert(variable_name, value);

    Ok(env)
}
fn eval_term(env: &Environment, term: Box<Term>) -> Result<Value> {
    Ok(match term.as_ref() {
        Term::String(s) => Value::String(s.clone()),
        Term::Integer(n) => Value::Number(*n),
        Term::Boolean(b) => Value::Boolean(*b),
        Term::Variable(s) => {
            debug!("eval_term: variable {s:?} found in env {:?}", env);
            let value = env.get(s).context("variable not found")?;
            value.clone()
        }
        Term::VariableIndexed(s, expr) => {
            let base_array = env.get(s).context("variable not found")?;
            let index = eval_expr(env, expr.clone())?;
            if let (Value::Number(n), Value::String(s)) = (index.clone(), base_array.clone()) {
                let ret = s
                    .chars()
                    .nth(n as usize)
                    .context("variableIndexed: index out of bounds")?;
                Value::String(ret.to_string())
            } else {
                bail!("Error: base_array : {base_array:?} is not a string or index : {index:?} is not a number")
            }
        }
    })
}
fn eval_expr(env: &Environment, expr: Box<Expr>) -> Result<Value> {
    match expr.as_ref().clone() {
        Add(left, right) => {
            let left = eval_term(env, left)?;
            let right = eval_term(env, right)?;
            match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::String(l), Value::Number(r)) => {
                    Ok(Value::Number(l.parse::<i64>().unwrap() + r))
                }
                (Value::Number(l), Value::String(r)) => {
                    Ok(Value::Number(l + r.parse::<i64>().unwrap()))
                }
                _ => bail!("Error: Addition of non-numbers"),
            }
        }
        Multiply(left, right) => {
            let left = eval_term(env, left)?;
            let right = eval_term(env, right)?;
            match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                (Value::String(l), Value::Number(r)) => {
                    Ok(Value::Number(l.parse::<i64>().unwrap() * r))
                }
                (Value::Number(l), Value::String(r)) => {
                    Ok(Value::Number(l * r.parse::<i64>().unwrap()))
                }
                _ => bail!("Error: Multiplication of non-numbers"),
            }
        }
        Expr::Equality(left, right) => {
            let left = eval_term(env, left)?;
            let right = eval_term(env, right)?;
            match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l == r)),
                (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l == r)),
                _ => bail!("Error: DisEquality of non-numbers"),
            }
        }
        Expr::LessThan(left, right) => {
            let left = eval_term(env, left)?;
            let right = eval_term(env, right)?;
            match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
                _ => bail!("Error: DisEquality of non-numbers"),
            }
        }
        DisEquality(left, right) => {
            let left = eval_term(env, left)?;
            let right = eval_term(env, right)?;
            match (left.clone(), right.clone()) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l != r)),
                (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l != r)),
                (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l != r)),
                _ => bail!("Error: DisEquality not implemented for: {left:?},{right:?}"),
            }
        }
        ContainedIn(left, right) => {
            let left = eval_term(env, left)?;
            let right = eval_term(env, right)?;
            match (left, right) {
                (Value::String(l), Value::String(r)) => Ok(Value::Boolean(r.contains(&l))),
                _ => bail!("Error: ContainedIn of non-strings"),
            }
        }
        TermWrapper(term) => eval_term(env, Box::new(term)),
        expr => bail!("eval_expr: unimplemented {expr:?}"),
    }
}
fn eval_print(env: Environment, expr: Box<Expr>) -> Result<Environment> {
    let value = eval_expr(&env, expr)?;
    match value {
        Value::String(s) => println!("{s}"),
        Value::Number(n) => println!("{n}"),
        Value::Boolean(b) => println!("{b}"),
        _ => unimplemented!("{value:?}"),
    }
    Ok(env)
}

fn eval_if(env: Environment, expr: Box<Expr>, body: Box<Statement>) -> Result<Environment> {
    Ok(if eval_expr(&env, expr)? == Value::Boolean(true) {
        eval(env, *body)?
    } else {
        env
    })
}
fn eval(env: Environment, expr: Statement) -> Result<Environment> {
    let ret = match expr {
        Statement::Assignment(variable_name, expr, is_let) => {
            evaluate_assignment(env, variable_name, expr, is_let)?
        }
        Statement::Print(expr) => eval_print(env, expr)?,
        Statement::If(expr, body) => eval_if(env, expr, body)?,
        Statement::While(expr, body) => {
            let mut env = env;
            while eval_expr(&env, expr.clone())? == Value::Boolean(true) {
                env = eval(env, *body.clone())?;
            }
            env
        }
        Statement::Block(block) => {
            let mut env = env;
            for expr in block {
                env = eval(env, expr)?;
            }
            env
        }
        _ => unimplemented!("{expr:?}"),
    };
    Ok(ret)
}
fn inner_run(program: Vec<Statement>) -> Result<Environment> {
    let mut env: Environment = HashMap::new();
    for expr in program {
        env = eval(env, expr)?;
    }
    Ok(env)
}

pub fn run(program: Vec<Statement>) -> Result<()> {
    inner_run(program)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_eval() {
        let program = vec![
            Statement::Assignment(
                "a".to_string(),
                Box::new(TermWrapper(Term::Integer(1))),
                true,
            ),
            Statement::Assignment(
                "b".to_string(),
                Box::new(TermWrapper(Term::Integer(2))),
                true,
            ),
        ];
        let env = inner_run(program).unwrap();
        let mut expected_env = HashMap::new();
        expected_env.insert("a".to_string(), Value::Number(1));
        expected_env.insert("b".to_string(), Value::Number(2));
        assert_eq!(env, expected_env);
    }

    #[test]
    fn test_simple() {
        let simple = r#"
let quiz_input := "
1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet
";
let sum := 0;
let index := 0;

while index < 42 {
    let is_first_digit_found := false;
    let first_digit_found := 0;
    let last_digit_found := 0;
    while quiz_input[index] != "\n" {
        if quiz_input[index] in "0123456789" {
            if is_first_digit_found == false {
                first_digit_found := quiz_input[index];
                is_first_digit_found := true;
            } 
            last_digit_found := quiz_input[index];
        }
        index := index + 1;
    }
    let first_digit_found_mult := first_digit_found * 10;
    sum := sum + first_digit_found_mult;
    sum := sum + last_digit_found;
    index := index + 1;
}
print sum;
"#;
        let tokens = crate::lexer::parse(simple).unwrap();
        let program = crate::parser::parse_input(tokens).unwrap();
        let env = inner_run(program).unwrap();
        if let Value::Number(n) = env.get("sum").unwrap() {
            assert_eq!(n, &142);
        } else {
            panic!("sum is not a number");
        }
    }
}
