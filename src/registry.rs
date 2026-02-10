use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use crate::dlfcn::{DlOpenFlags, DlSym, DynLib};

const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[m";

static DLIBS: OnceLock<Mutex<HashMap<String, DynLib>>> = OnceLock::new();

fn dlibs() -> &'static Mutex<HashMap<String, DynLib>> {
    DLIBS.get_or_init(|| {
        let mut hm = HashMap::new();
        hm.insert(
            "libc.so.6".to_string(),
            DynLib::open("libc.so.6", &[DlOpenFlags::RTLD_LAZY]).unwrap(),
        );

        Mutex::new(hm)
    })
}

pub fn add_lib(libname: &str) {
    let lib = DynLib::open(libname, &[DlOpenFlags::RTLD_LAZY]);
    match lib {
        Ok(lib) => {
            dlibs().lock().unwrap().insert(libname.to_string(), lib);
        }
        Err(e) => {
            eprintln!("{RED}{}{RESET}", e);
        }
    }
}

pub fn del_lib(libname: &str) {
    if dlibs().lock().unwrap().remove(libname).is_none() {
        eprintln!("{RED}The library {libname} was not linked to unlink{RESET}");
    }
}

pub fn get_libs() {
    println!("INFO: Listing linked libraries: ");
    let libs = dlibs().lock().unwrap();
    for (libname, _) in libs.iter() {
        println!("\t- {libname}");
    }
}

pub fn get_sym(sym: &str) -> Option<DlSym> {
    let libs = dlibs().lock().unwrap();
    let mut lookedup_libs = Vec::new();
    for (libname, lib) in libs.iter() {
        match DlSym::new(lib, sym) {
            Ok(dlsym) => {
                return Some(dlsym);
            }
            Err(_) => {
                lookedup_libs.push(libname);
            }
        }
    }
    eprintln!(
        "{RED}Could not find the symbol `{sym}`, try linking it from a shared object, we looked up the following shared objects:{RESET}"
    );
    lookedup_libs
        .iter()
        .for_each(|l| eprintln!("\t{RED}- {l}{RESET}"));

    None
}
