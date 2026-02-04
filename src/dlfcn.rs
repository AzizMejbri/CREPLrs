use std::{
    error::Error,
    ffi::{CString, c_char, c_int, c_void},
    fmt::{self, Debug, Display, Formatter},
    result::Result,
};

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub enum DlOpenFlags {
    RTLD_LAZY = 1,
    RTLD_NOW = 2,
    RTLD_GLOBAL = 0x100,
    RTLD_LOCAL = 0,
    RTLD_NODELETE = 0x1000,
    RTLD_NOLOAD = 0x4,
    RTLD_DEEPBIND = 0x8,
}

unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlsym(handle: *const c_void, symbol: *const c_char) -> *mut c_void;
}

pub struct DlError(String);
impl Display for DlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DL error: {}", self.0)
    }
}
impl Debug for DlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DL error: {}", self.0)
    }
}
impl Error for DlError {}

impl DlError {
    fn new(msg: &str) -> Self {
        Self(msg.to_string())
    }
}

#[derive(Debug)]
pub struct DynLib {
    handle: *mut c_void,
}

impl DynLib {
    pub fn open(filename: &str, flags: &[DlOpenFlags]) -> Result<Self, DlError> {
        let c_filename = CString::new(filename)
            .map_err(|e| DlError::new(&format!("Error {}, Invalid Filename {}", e, filename)));
        let combined_flags = flags.iter().fold(0, |acc, flag| acc | *flag as c_int);
        let handle = unsafe { dlopen(c_filename.unwrap().as_ptr(), combined_flags) };
        if handle.is_null() {
            return Err(DlError(format!(
                "Error Opening the shared object: {}",
                filename
            )));
        }
        Ok(Self { handle })
    }

    pub fn close(&self) -> Result<(), DlError> {
        if unsafe { dlclose(self.handle) } == 0 {
            return Ok(());
        }
        Err(DlError(format!(
            "Error closing the shared library: {:?}",
            self
        )))
    }
}

#[derive(Debug)]
pub struct DlSym {
    fn_ptr: *mut c_void,
}

impl DlSym {
    pub fn new(lib: &DynLib, symbol: &str) -> Result<Self, DlError> {
        let c_sym = CString::new(symbol)
            .map_err(|e| DlError(format!("Error {}: Invalid Symbol name `{}`", e, symbol)))?;
        let found_sym = unsafe { dlsym(lib.handle, c_sym.as_ptr()) };
        if found_sym.is_null() {
            return Err(DlError("Could not find the symbol".to_string()));
        }
        Ok(Self { fn_ptr: found_sym })
    }
}

impl From<DlSym> for *mut c_void {
    fn from(value: DlSym) -> Self {
        value.fn_ptr
    }
}

impl Drop for DynLib {
    fn drop(&mut self) {
        unsafe { dlclose(self.handle) };
    }
}
