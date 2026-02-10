use crate::cffi::{CallInterface, FfiType};
use std::{
    error::Error,
    ffi::{CStr, c_void},
};

use CREPLrs::{
    cffi::{self},
    cli::{Cli, OpMode},
    lex::Token,
    registry::{add_lib, del_lib, get_libs, get_sym},
};

use libc::{FILE, c_char};
unsafe extern "C" {
    static mut stdout: *mut FILE;
    fn setvbuf(stream: *mut FILE, buf: *mut libc::c_char, mode: i32, size: usize) -> i32;
}

const RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[m";

fn main() -> Result<(), Box<dyn Error>> {
    let mut mode = OpMode::Void;
    let mut last = String::new();
    let mut cli = Cli::new(&mode);
    unsafe {
        setvbuf(stdout, std::ptr::null_mut(), libc::_IONBF, 0);
    }
    while let Some(tokens) = cli.next() {
        if tokens.is_empty() {
            continue;
        }
        if tokens[0].1 == ":r" {
            println!("{last}");
            continue;
        }
        if tokens[0].1 == ":d" {
            mode = OpMode::Int;
            cli.update_mode(&mode);
            continue;
        }
        if tokens[0].1 == ":f" {
            mode = OpMode::Float;
            cli.update_mode(&mode);
            continue;
        }
        if tokens[0].1 == ":c" {
            mode = OpMode::Char;
            cli.update_mode(&mode);
            continue;
        }
        if tokens[0].1 == ":v" {
            mode = OpMode::Void;
            cli.update_mode(&mode);
            continue;
        }
        if tokens[0].1 == ":s" {
            mode = OpMode::Ptr;
            cli.update_mode(&mode);
            continue;
        }
        if tokens[0].1 == ":l" {
            tokens.iter().skip(1).for_each(|tok| match tok.0 {
                Token::FileName => add_lib(&tok.1),
                _ => eprintln!("{RED}`{}` is not a valid file name!{RESET}", tok.1),
            });
            continue;
        }
        if tokens[0].1 == ":ul" {
            tokens.iter().skip(1).for_each(|tok| del_lib(&tok.1));
            continue;
        }
        if tokens[0].1 == ":ls" {
            get_libs();
            continue;
        }
        if tokens[0].0 != Token::Id {
            eprintln!("{RED}ERROR: Expected a function as the first lexeme{RESET}");
            continue;
        }
        let maybe_called_fn = get_sym(&tokens[0].1);
        #[allow(clippy::needless_late_init)]
        let called_fn;
        match maybe_called_fn {
            Some(f) => {
                called_fn = f;
            }
            None => {
                continue;
            }
        }
        let mut cif_arg_types = Vec::new();
        let mut cif_args = Vec::new();
        let mut arg_boxes: Vec<Box<dyn std::any::Any>> = Vec::new();
        for token in tokens.iter().skip(1) {
            cif_arg_types.push(match token.0 {
                Token::CString => FfiType::Pointer,
                Token::CInt => FfiType::SInt64,
                Token::CFloat => FfiType::Double,
                Token::CChar => FfiType::SInt8,
                _ => FfiType::Void,
            });
            match token.0 {
                Token::CString => {
                    let cstr = Box::new(std::ffi::CString::new(token.1.clone()).unwrap());
                    let char_ptr = cstr.as_ptr();
                    arg_boxes.push(cstr);

                    let ptr_box = Box::new(char_ptr);
                    cif_args.push(&*ptr_box as *const _ as *mut c_void);
                    arg_boxes.push(ptr_box);
                }
                Token::CInt => {
                    let val = Box::new(token.1.parse::<i64>().unwrap());
                    cif_args.push(&*val as *const _ as *mut c_void);
                    arg_boxes.push(val);
                }
                Token::CFloat => {
                    let val = Box::new(token.1.parse::<f64>().unwrap());
                    cif_args.push(&*val as *const _ as *mut c_void);
                    arg_boxes.push(val);
                }
                Token::CChar => {
                    let ch = token.1.as_bytes()[0] as i8;
                    let val = Box::new(ch);
                    cif_args.push(&*val as *const _ as *mut c_void);
                    arg_boxes.push(val);
                }
                _ => {}
            }
        }
        match mode {
            OpMode::Int => {
                let mut cif = CallInterface::<i64>::new(cif_arg_types)?;
                let res = cif.call(called_fn, &cif_args);
                last = format!("{}", res);
                println!("\n{BLUE}{last}{RESET}");
            }
            OpMode::Void => {
                let mut cif = CallInterface::<()>::new(cif_arg_types)?;
                cif.call(called_fn, &cif_args);
                last = format!("()");
            }
            OpMode::Float => {
                let mut cif = CallInterface::<f64>::new(cif_arg_types)?;
                let res = cif.call(called_fn, &cif_args);
                last = format!("{}", res);
                println!("\n{BLUE}{last}{RESET}");
            }
            OpMode::Ptr => {
                let mut cif = CallInterface::<*const c_char>::new(cif_arg_types)?;
                let res = cif.call(called_fn, &cif_args);
                if res.is_null() {
                    last = "(NullString)".to_string();
                    println!("\n{BLUE}{last}{RESET}");
                } else {
                    unsafe {
                        last = format!("{}", CStr::from_ptr(res).to_str().unwrap());
                        println!("\n{BLUE}{last}{RESET}");
                    }
                }
            }
            OpMode::Char => {
                let mut cif = CallInterface::<i8>::new(cif_arg_types)?;
                let res = cif.call(called_fn, &cif_args);
                last = format!("{}", res);
                println!("\n{BLUE}{last}{RESET}");
            }
        }
    }
    Ok(())
}
