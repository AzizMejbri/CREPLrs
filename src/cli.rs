use std::io::Write;

use rustyline::{Config, Editor, history::DefaultHistory};

use crate::lex::{Token, lex};

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[m";

#[derive(Debug, Default, Clone)]
pub enum OpMode {
    Float,
    #[default]
    Int,
    Ptr,
    Char,
    Void,
}

#[derive(Default)]
pub struct Cli {
    mode: OpMode,
    counter: i32,
    rl: Option<Editor<(), DefaultHistory>>,
}

impl Cli {
    pub fn new(mode: &OpMode) -> Self {
        let mut rl = Editor::with_config(
            Config::builder()
                .history_ignore_space(true)
                .max_history_size(1000)
                .unwrap()
                .build(),
        )
        .unwrap_or_else(|_| Editor::new().unwrap());
        let _ = rl.load_history("hist.txt");
        Self {
            mode: mode.clone(),
            counter: 0,
            rl: Some(rl),
        }
    }
    pub fn update_mode(&mut self, mode: &OpMode) {
        self.mode = mode.clone();
    }
    pub fn editor(&mut self) -> &mut Editor<(), DefaultHistory> {
        if self.rl.is_none() {
            let mut rl = Editor::new().unwrap();
            let _ = rl.load_history("history.txt");
            self.rl = Some(rl);
        }
        self.rl.as_mut().unwrap()
    }
}

impl Iterator for Cli {
    type Item = Vec<(Token, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter >= 1000 {
            return None;
        }
        let mode_text = format!("{GREEN}[{:?}] {RESET}$ ", self.mode);
        let readline = self.editor().readline(&mode_text);

        match readline {
            Ok(line) => {
                let _ = self.editor().add_history_entry(&line);
                self.counter += 1;
                if self.counter & 7 == 0 {
                    let _ = self.editor().save_history("hist.txt");
                }
                let tokens = lex(&line);
                return Some(tokens);
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                eprintln!("to exit press CTRL+D");
                return Some(Vec::new());
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("\nGoodbye!");
                let _ = self.editor().save_history("hist.txt");
                return None;
            }
            Err(e) => {
                eprintln!("{RED}Cli Error found: {e}{RESET}");
                return None;
            }
        }
    }
}

impl Clone for Cli {
    fn clone(&self) -> Self {
        Cli {
            mode: self.mode.clone(),
            counter: self.counter,
            rl: None,
        }
    }
}
