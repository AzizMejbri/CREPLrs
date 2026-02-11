use std::collections::HashMap;

use crate::parser::{BinaryOp, Expr, UnaryOp};

#[derive(Debug, Clone)]
pub enum Value {
    CString(String),
    CChar(char),
    Number(f64),
    Integer(i64),
    Bool(bool),
}

#[derive(Clone, Debug)]
pub struct Env {
    pub vars: HashMap<String, Value>,
    pub consts: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            consts: HashMap::new(),
        }
    }
    pub fn display(&self) {
        println!("{:?}", self)
    }

    pub fn set_var(&mut self, name: String, value: Value) -> Result<(), String> {
        if self.consts.contains_key(&name) {
            return Err(format!("'{}' is a constant, cannot reassign", name));
        }
        self.vars.insert(name, value);
        Ok(())
    }

    pub fn set_const(&mut self, name: String, value: Value) -> Result<(), String> {
        if self.consts.contains_key(&name) || self.vars.contains_key(&name) {
            return Err(format!("'{}' already defined", name));
        }
        self.consts.insert(name, value);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.consts
            .get(name)
            .or_else(|| self.vars.get(name))
            .cloned()
    }
}

pub fn eval(expr: &Expr, env: &Env) -> Result<Value, String> {
    match expr {
        Expr::Integer(i) => Ok(Value::Integer(*i)),
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::CChar(c) => Ok(Value::CChar(*c)),
        Expr::CString(s) => Ok(Value::CString(s.clone())),
        Expr::Variable(name) => env
            .get(name)
            .ok_or_else(|| format!("Undefined variable: '{}'", name)),
        Expr::Unary(op, expr) => {
            let val = eval(expr, env)?;
            match op {
                UnaryOp::Neg => match val {
                    Value::Integer(i) => Ok(Value::Integer(-i)),
                    Value::Number(n) => Ok(Value::Number(-n)),
                    _ => Err("Cannot negate this type".to_string()),
                },
                UnaryOp::Not => {
                    let b = match val {
                        Value::Bool(b) => b,
                        Value::Integer(i) => i != 0,
                        Value::Number(n) => n != 0.0,
                        _ => {
                            eprintln!("\x1b[31mSyntax Error: !<string>\x1b[m");
                            false
                        }
                    };
                    Ok(Value::Bool(!b))
                }
            }
        }
        Expr::Binary(left, op, right) => {
            let left_val = eval(left, env)?;
            let right_val = eval(right, env)?;

            match op {
                BinaryOp::Add => match (left_val, right_val) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                    (Value::Integer(a), Value::Number(b)) => Ok(Value::Number(a as f64 + b)),
                    (Value::Number(a), Value::Integer(b)) => Ok(Value::Number(a + b as f64)),
                    (Value::CString(a), Value::CString(b)) => Ok(Value::CString({
                        let mut out = a;
                        out.push_str(&b);
                        out
                    })),
                    (Value::CString(a), Value::CChar(b)) => Ok(Value::CString({
                        let mut out = a;
                        out.push(b);
                        out
                    })),
                    (Value::CChar(a), Value::CString(b)) => Ok(Value::CString({
                        let mut out = a.to_string();
                        out.push_str(&b);
                        out
                    })),
                    (Value::CChar(a), Value::CChar(b)) => Ok(Value::CString({
                        let mut out = a.to_string();
                        out.push(b);
                        out
                    })),
                    _ => Err("Cannot add these types".to_string()),
                },
                BinaryOp::Sub => match (left_val, right_val) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                    (Value::Integer(a), Value::Number(b)) => Ok(Value::Number(a as f64 - b)),
                    (Value::Number(a), Value::Integer(b)) => Ok(Value::Number(a - b as f64)),
                    _ => Err("Cannot subtract these types".to_string()),
                },
                BinaryOp::Mul => match (left_val, right_val) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                    (Value::Integer(a), Value::Number(b)) => Ok(Value::Number(a as f64 * b)),
                    (Value::Number(a), Value::Integer(b)) => Ok(Value::Number(a * b as f64)),
                    (Value::CString(a), Value::Integer(b)) => {
                        Ok(Value::CString(a.repeat(b as usize)))
                    }
                    (Value::CChar(a), Value::Integer(b)) => {
                        Ok(Value::CString(a.to_string().repeat(b as usize)))
                    }
                    _ => Err("Cannot multiply these types".to_string()),
                },
                BinaryOp::Div => match (left_val, right_val) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if b == 0 {
                            Err("Division by zero".to_string())
                        } else {
                            // Return float for division to match C behavior
                            Ok(Value::Number(a as f64 / b as f64))
                        }
                    }
                    (Value::Number(a), Value::Number(b)) => {
                        if b == 0.0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(Value::Number(a / b))
                        }
                    }
                    _ => Err("Cannot divide these types".to_string()),
                },
                BinaryOp::Eq => {
                    let eq = match (&left_val, &right_val) {
                        (Value::Integer(a), Value::Integer(b)) => a == b,
                        (Value::Number(a), Value::Number(b)) => a == b,
                        (Value::Bool(a), Value::Bool(b)) => a == b,
                        (Value::CString(a), Value::CString(b)) => a == b,
                        _ => false,
                    };
                    Ok(Value::Bool(eq))
                }
                BinaryOp::Ne => {
                    let eq = match (&left_val, &right_val) {
                        (Value::Integer(a), Value::Integer(b)) => a != b,
                        (Value::Number(a), Value::Number(b)) => a != b,
                        (Value::Bool(a), Value::Bool(b)) => a != b,
                        (Value::CString(a), Value::CString(b)) => a != b,
                        _ => false,
                    };
                    Ok(Value::Bool(!eq))
                }
                BinaryOp::Lt => match (left_val, right_val) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a < b)),
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
                    _ => Err("Cannot compare these types".to_string()),
                },
                BinaryOp::Le => match (left_val, right_val) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a <= b)),
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
                    _ => Err("Cannot compare these types".to_string()),
                },
                BinaryOp::Gt => match (left_val, right_val) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a > b)),
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
                    _ => Err("Cannot compare these types".to_string()),
                },
                BinaryOp::Ge => match (left_val, right_val) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a >= b)),
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
                    _ => Err("Cannot compare these types".to_string()),
                },
            }
        }
    }
}
