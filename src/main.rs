use crate::cffi::{CallInterface, FfiType};
use CREPLrs::ffi_call;
use std::{
    error::Error,
    ffi::c_void,
    io::{self, Write},
};

use CREPLrs::{
    cffi::{self},
    cli::Cli,
    dlfcn::{self, DlSym},
    lex::Token,
};

use libc::FILE;
unsafe extern "C" {
    fn fflush(stream: *mut FILE) -> i32;
}

unsafe extern "C" {
    static mut stdout: *mut FILE;
    fn setvbuf(stream: *mut FILE, buf: *mut libc::c_char, mode: i32, size: usize) -> i32;
}

fn main() -> Result<(), Box<dyn Error>> {
    let lib = dlfcn::DynLib::open("libraylib.so", &[dlfcn::DlOpenFlags::RTLD_LAZY]).unwrap();
    let cli = Cli::new();
    unsafe {
        setvbuf(stdout, std::ptr::null_mut(), libc::_IONBF, 0);
    }
    for tokens in cli {
        if tokens.is_empty() {
            continue;
        }
        if tokens[0].0 != Token::Id {
            eprintln!("Error: Expected a function as the first lexeme");
            continue;
        }
        let maybe_called_fn = DlSym::new(&lib, &tokens[0].1);
        let called_fn;
        match maybe_called_fn {
            Ok(f) => {
                called_fn = f;
            }
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        }
        // println!("{called_fn:?}");
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
        let mut cif = CallInterface::<()>::new(cif_arg_types)?;
        let res = cif.call(called_fn, &cif_args);
    }
    Ok(())
}
