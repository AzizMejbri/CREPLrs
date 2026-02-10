use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Id,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*(\.[a-zA-Z0-9_]+)+")]
    FileName,

    #[regex(r#""([^"\\]|\\.)*""#)]
    CString,

    #[regex(r"[-+]?([0-9]*\.[0-9]+|[0-9]+\.[0-9]*)([eE][+-]?[0-9]+)?|-?[0-9]+[eE][+-]?[0-9]+")]
    CFloat,

    #[regex(r"[-+]?(0[xX][0-9a-fA-F]+|0[bB][01]+|0[0-7]*|[1-9][0-9]*|0)")]
    CInt,

    #[regex(r"'.'")]
    CChar,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,

    #[token("==")]
    EqEq,
    #[token("!=")]
    BangEq,
    #[token("<=")]
    Le,
    #[token("<")]
    Lt,
    #[token(">=")]
    Ge,
    #[token(">")]
    Gt,

    #[token("!")]
    Bang,

    #[regex(r"[ \t\n\f]+", logos::skip)]
    WS,

    #[regex(r":[rcdfvslp]|:ul|:ls|:const|:var|:t|:pa")]
    Command,
}

#[allow(unused)]
pub fn lex(cmd: &str) -> Vec<(Token, String)> {
    let mut lexer = Token::lexer(cmd);
    let mut out = Vec::new();
    while let Some(token) = lexer.next() {
        match token {
            Err(_) => {
                println!("Unrecognized token!");
            }
            Ok(Token::WS) => {}
            Ok(Token::CString) => {
                out.push((Token::CString, lexer.slice().trim_matches('"').to_string()));
            }
            Ok(Token::CChar) => out.push((
                Token::CChar,
                lexer.slice().chars().nth(1).unwrap().to_string(),
            )),
            Ok(tok) => {
                out.push((tok, lexer.slice().to_string()));
            }
        }
    }
    out
}
