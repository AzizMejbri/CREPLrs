use crate::lex::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Integer(i64),
    CString(String),
    Variable(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg, // -
    Not, // !
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

pub struct Parser {
    tokens: Vec<(Token, String)>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, String)>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&(Token, String)> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let (token, text) = self
            .tokens
            .get(self.pos)
            .ok_or("Unexpected end of input")?
            .clone();

        self.pos += 1;

        match token {
            Token::CInt => {
                let value = if text.starts_with("0x") || text.starts_with("0X") {
                    i64::from_str_radix(&text[2..], 16)
                } else if text.starts_with("0b") || text.starts_with("0B") {
                    i64::from_str_radix(&text[2..], 2)
                } else if text.starts_with('0') && text.len() > 1 {
                    i64::from_str_radix(&text[1..], 8)
                } else {
                    text.parse::<i64>()
                }
                .map_err(|_| format!("Invalid integer: {}", text))?;

                Ok(Expr::Integer(value))
            }
            Token::CFloat => {
                let value = text
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid float: {}", text))?;
                Ok(Expr::Number(value))
            }
            Token::Id => Ok(Expr::Variable(text.to_string())),
            Token::CString => Ok(Expr::Variable(text.to_string())),
            Token::LParen => {
                let expr = self.parse_expr(0)?;
                match self.peek() {
                    Some((Token::RParen, _)) => {
                        self.advance();
                        Ok(expr)
                    }
                    _ => Err("Expected ')'".to_string()),
                }
            }
            Token::Minus => {
                let expr = self.parse_primary()?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(expr)))
            }
            Token::Bang => {
                let expr = self.parse_primary()?;
                Ok(Expr::Unary(UnaryOp::Not, Box::new(expr)))
            }
            _ => Err(format!("Unexpected token: {:?}", token)),
        }
    }

    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_primary()?;

        while let Some((token, _)) = self.peek() {
            let (op, prec, assoc) = match token {
                Token::Plus => (BinaryOp::Add, 1, Assoc::Left),
                Token::Minus => (BinaryOp::Sub, 1, Assoc::Left),
                Token::Star => (BinaryOp::Mul, 2, Assoc::Left),
                Token::Slash => (BinaryOp::Div, 2, Assoc::Left),
                Token::EqEq => (BinaryOp::Eq, 0, Assoc::Left),
                Token::BangEq => (BinaryOp::Ne, 0, Assoc::Left),
                Token::Lt => (BinaryOp::Lt, 0, Assoc::Left),
                Token::Le => (BinaryOp::Le, 0, Assoc::Left),
                Token::Gt => (BinaryOp::Gt, 0, Assoc::Left),
                Token::Ge => (BinaryOp::Ge, 0, Assoc::Left),
                _ => break,
            };

            if prec < min_prec {
                break;
            }

            self.advance();
            let next_min_prec = match assoc {
                Assoc::Left => prec + 1,
                Assoc::Right => prec,
            };
            let rhs = self.parse_expr(next_min_prec)?;
            lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
        }

        Ok(lhs)
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        self.parse_expr(0)
    }
}

#[derive(Debug, Clone, Copy)]
enum Assoc {
    Left,
    Right,
}
