use crate::{lex::Token, parser::Parser};

use crate::eval::{Env, Value, eval};

use chumsky::container::Seq;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static GLOBAL_ENV: Lazy<Mutex<Env>> = Lazy::new(|| {
    let mut env = Env::new();
    env.set_const("PI".to_string(), Value::Number(std::f64::consts::PI))
        .unwrap();
    env.set_const("E".to_string(), Value::Number(std::f64::consts::E))
        .unwrap();
    env.set_const("TRUE".to_string(), Value::Bool(true))
        .unwrap();
    env.set_const("FALSE".to_string(), Value::Bool(false))
        .unwrap();
    Mutex::new(env)
});

pub fn const_eval(tokens: Vec<(Token, String)>) -> Result<String, String> {
    if tokens.is_empty() {
        return Err("Usage: :const <name> <expression>".to_string());
    }

    // First token is the constant name
    let name = match &tokens[0] {
        (Token::Id, name) => name.clone(),
        _ => return Err("Constant name must be an identifier".to_string()),
    };

    let mut parser = Parser::new(tokens[1..].to_vec());
    let expr = parser.parse()?;

    let mut env = GLOBAL_ENV.lock().unwrap();

    let value = eval(&expr, &env)?;

    env.set_const(name.clone(), value)?;

    Ok(format!("Constant '{}' defined", name))
}

pub fn var_eval(tokens: Vec<(Token, String)>) -> Result<String, String> {
    if tokens.is_empty() {
        return Err("Usage: :var <name> <expression>".to_string());
    }

    let name = match &tokens[0] {
        (Token::Id, name) => name.clone(),
        _ => return Err("Variable name must be an identifier".to_string()),
    };

    let mut parser = Parser::new(tokens[1..].to_vec());
    let expr = parser.parse()?;

    let mut env = GLOBAL_ENV.lock().unwrap();

    let value = eval(&expr, &env)?;

    env.set_var(name.clone(), value)?;

    Ok(format!("Variable '{}' set", name))
}

const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[m";

pub fn display_vars(tokens: Vec<(Token, String)>) {
    tokens.iter().for_each(|tok| match tok.0 {
        Token::Id => {
            println!(
                "\t- {} -> {:?}",
                tok.1,
                GLOBAL_ENV.lock().unwrap().get(&tok.1)
            );
        }
        _ => {
            eprintln!(
                "{RED}\t- Error: `{}` was expected to be an identifier {RESET}",
                tok.1
            )
        }
    });
}

pub fn display_all() {
    let env = GLOBAL_ENV.lock().unwrap();
    let vars = env.vars.clone();
    let consts = env.consts.clone();
    println!("vars: ");
    for (name, value) in vars.iter() {
        println!("\t- {} -> {:?}", name, value)
    }

    println!("\nconsts: ");
    for (name, value) in consts.iter() {
        println!("\t- {} -> {:?}", name, value)
    }
}
