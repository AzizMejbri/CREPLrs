use std::io::Write;

use crate::lex::{Token, lex};

unsafe extern "C" {
    fn fflush(stream: *mut std::ffi::c_void) -> i32;
}

unsafe extern "C" {
    static mut stdout: *mut std::ffi::c_void;
}

#[derive(Default)]
pub struct Cli {
    counter: i32,
}

impl Cli {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

impl Iterator for Cli {
    type Item = Vec<(Token, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            fflush(stdout);
        }
        if self.counter >= 20 {
            return None;
        }
        let mut cmd = String::new();
        std::io::stdout().lock().write_all(b"$ ").unwrap();
        std::io::stdout().flush().unwrap();
        let bytes_read = std::io::stdin().read_line(&mut cmd).unwrap();
        if bytes_read == 0 {
            // Ctrl + D
            println!("GoodBye!");
            return None;
        }
        let tokens = lex(&cmd);
        Some(tokens)
    }
}
