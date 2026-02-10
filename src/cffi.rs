use std::{
    error::Error,
    ffi::c_void,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    mem,
};

pub type ABI = u32;
pub type Status = i32;

pub const FFI_UNIX64: ABI = 2;
pub const FFI_DEFAULT_ABI: ABI = FFI_UNIX64;
pub const FFI_OK: Status = 0;

#[repr(C)]
pub struct ffi_type {
    pub size: usize,
    pub alignment: u16,
    pub type_: u16,
    pub elements: *mut *mut ffi_type,
}

#[repr(C)]
#[derive(Debug)]
pub struct ffi_cif {
    pub abi: ABI,
    pub nargs: u32,
    pub arg_types: *mut *mut ffi_type,
    pub rtype: *mut ffi_type,
    pub bytes: u32,
    pub flags: u32,
}

#[link(name = "ffi")]
unsafe extern "C" {
    // Common ffi_type constants (these would be defined in libffi)
    pub static mut ffi_type_void: ffi_type;
    pub static mut ffi_type_sint8: ffi_type;
    pub static mut ffi_type_sint16: ffi_type;
    pub static mut ffi_type_sint32: ffi_type;
    pub static mut ffi_type_sint64: ffi_type;
    pub static mut ffi_type_uint8: ffi_type;
    pub static mut ffi_type_uint16: ffi_type;
    pub static mut ffi_type_uint32: ffi_type;
    pub static mut ffi_type_uint64: ffi_type;
    pub static mut ffi_type_float: ffi_type;
    pub static mut ffi_type_double: ffi_type;
    pub static mut ffi_type_pointer: ffi_type;

    pub fn ffi_prep_cif(
        cif: *mut ffi_cif,
        abi: ABI,
        nargs: u32,
        rtype: *mut ffi_type,
        atypes: *mut *mut ffi_type,
    ) -> Status;

    pub fn ffi_call(
        cif: *mut ffi_cif,
        fn_: *mut c_void,
        rvalue: *mut c_void,
        avalue: *mut *mut c_void,
    );
}

#[derive(Debug)]
pub struct FfiError(String);

impl Display for FfiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "FFI error: {}", self.0)
    }
}

impl Error for FfiError {}

#[allow(dead_code)]
impl FfiError {
    fn new(msg: &str) -> Self {
        Self(msg.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FfiType {
    Void,
    SInt8,
    SInt16,
    SInt32,
    SInt64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
    Pointer,
}

impl FfiType {
    pub fn raw(&self) -> *mut ffi_type {
        unsafe {
            match self {
                FfiType::Void => &raw mut ffi_type_void as *mut _,
                FfiType::SInt8 => &raw mut ffi_type_sint8 as *mut _,
                FfiType::SInt16 => &raw mut ffi_type_sint16 as *mut _,
                FfiType::SInt32 => &raw mut ffi_type_sint32 as *mut _,
                FfiType::SInt64 => &raw mut ffi_type_sint64 as *mut _,
                FfiType::UInt8 => &raw mut ffi_type_uint8 as *mut _,
                FfiType::UInt16 => &raw mut ffi_type_uint16 as *mut _,
                FfiType::UInt32 => &raw mut ffi_type_uint32 as *mut _,
                FfiType::UInt64 => &raw mut ffi_type_uint64 as *mut _,
                FfiType::Float => &raw mut ffi_type_float as *mut _,
                FfiType::Double => &raw mut ffi_type_double as *mut _,
                FfiType::Pointer => &raw mut ffi_type_pointer as *mut _,
            }
        }
    }
}
impl From<i8> for FfiType {
    fn from(_: i8) -> Self {
        FfiType::SInt8
    }
}

impl From<u8> for FfiType {
    fn from(_: u8) -> Self {
        FfiType::UInt8
    }
}

impl From<i32> for FfiType {
    fn from(_: i32) -> Self {
        FfiType::SInt32
    }
}

impl From<u32> for FfiType {
    fn from(_: u32) -> Self {
        FfiType::UInt32
    }
}

impl From<i64> for FfiType {
    fn from(_: i64) -> Self {
        FfiType::SInt64
    }
}

impl From<u64> for FfiType {
    fn from(_: u64) -> Self {
        FfiType::UInt64
    }
}

impl From<f32> for FfiType {
    fn from(_: f32) -> Self {
        FfiType::Float
    }
}

impl From<f64> for FfiType {
    fn from(_: f64) -> Self {
        FfiType::Double
    }
}

impl<T> From<*const T> for FfiType {
    fn from(_: *const T) -> Self {
        FfiType::Pointer
    }
}

impl<T> From<*mut T> for FfiType {
    fn from(_: *mut T) -> Self {
        FfiType::Pointer
    }
}

impl From<()> for FfiType {
    fn from(_: ()) -> Self {
        FfiType::Void
    }
}

pub struct CallInterface<R>
where
    R: Into<FfiType>,
{
    cif: ffi_cif,
    arg_types: Vec<FfiType>,
    arg_types_raw: Vec<*mut ffi_type>,
    ret_type: FfiType,
    phantom: std::marker::PhantomData<R>,
}

pub trait IntoFfiArg {
    fn into_ffi(self, boxes: &mut Vec<Box<dyn std::any::Any>>) -> *mut std::ffi::c_void;
}

impl IntoFfiArg for i8 {
    fn into_ffi(self, boxes: &mut Vec<Box<dyn std::any::Any>>) -> *mut std::ffi::c_void {
        let b = Box::new(self);
        let ptr = &*b as *const _ as *mut _;
        boxes.push(b);
        ptr
    }
}

impl IntoFfiArg for i64 {
    fn into_ffi(self, boxes: &mut Vec<Box<dyn std::any::Any>>) -> *mut std::ffi::c_void {
        let b = Box::new(self);
        let ptr = &*b as *const _ as *mut _;
        boxes.push(b);
        ptr
    }
}

impl IntoFfiArg for f64 {
    fn into_ffi(self, boxes: &mut Vec<Box<dyn std::any::Any>>) -> *mut std::ffi::c_void {
        let b = Box::new(self);
        let ptr = &*b as *const _ as *mut _;
        boxes.push(b);
        ptr
    }
}

impl IntoFfiArg for &'static str {
    fn into_ffi(self, boxes: &mut Vec<Box<dyn std::any::Any>>) -> *mut std::ffi::c_void {
        let cstr = Box::new(std::ffi::CString::new(self).unwrap());
        let char_ptr = cstr.as_ptr();
        boxes.push(cstr);

        // Box the pointer value itself
        let ptr_box = Box::new(char_ptr);
        let ptr_to_ptr = &*ptr_box as *const _ as *mut _;
        boxes.push(ptr_box);
        ptr_to_ptr
    }
}

impl IntoFfiArg for String {
    fn into_ffi(self, boxes: &mut Vec<Box<dyn std::any::Any>>) -> *mut std::ffi::c_void {
        let cstr = Box::new(std::ffi::CString::new(self).unwrap());
        let char_ptr = cstr.as_ptr();
        boxes.push(cstr);

        // Box the pointer value itself
        let ptr_box = Box::new(char_ptr);
        let ptr_to_ptr = &*ptr_box as *const _ as *mut _;
        boxes.push(ptr_box);
        ptr_to_ptr
    }
}

impl<T> IntoFfiArg for Box<T>
where
    T: IntoFfiArg,
{
    fn into_ffi(self, boxes: &mut Vec<Box<dyn std::any::Any>>) -> *mut std::ffi::c_void {
        (*self).into_ffi(boxes)
    }
}
impl<R> CallInterface<R>
where
    R: Into<FfiType>,
    R: Default,
{
    pub fn new<A>(arg_types: A) -> Result<Self, FfiError>
    where
        A: IntoIterator<Item = FfiType>,
    {
        let ret_type: FfiType = R::into(R::default());
        // println!("Return type: {:?}", ret_type);
        // println!("Return type raw: {:p}", ret_type.raw());

        let arg_types_vec: Vec<FfiType> = arg_types.into_iter().collect();
        // println!("Argument types: {:?}", arg_types_vec);
        // println!("Number of arguments: {}", arg_types_vec.len());

        let mut arg_types_raw_vec: Vec<*mut ffi_type> =
            arg_types_vec.iter().map(|t| t.raw()).collect();
        // println!("Raw argument pointers: {:?}", arg_types_raw_vec);

        let mut cif: ffi_cif = unsafe { mem::zeroed() };

        let result: Status = unsafe {
            ffi_prep_cif(
                &mut cif,
                FFI_DEFAULT_ABI,
                arg_types_raw_vec.len() as u32,
                ret_type.raw(),
                arg_types_raw_vec.as_mut_ptr(),
            )
        };
        // println!("ffi_prep_cif result: {}", result);

        if result != FFI_OK {
            return Err(FfiError(format!(
                "Error Preparing the CallInterface: C::ffi_prep_cif returned {}",
                result
            )));
        }

        Ok(Self {
            cif,
            arg_types: arg_types_vec,
            arg_types_raw: arg_types_raw_vec,
            ret_type,
            phantom: PhantomData,
        })
    }

    pub fn call<F>(&mut self, f: F, arg_values: &[*mut c_void]) -> R
    where
        R: Default,
        F: Into<*mut c_void>,
    {
        let mut result: R = Default::default();
        let fn_ptr: *mut c_void = f.into();
        unsafe {
            ffi_call(
                &mut self.cif,
                fn_ptr,
                if self.ret_type == FfiType::Void {
                    std::ptr::null_mut()
                } else {
                    &mut result as *mut _ as *mut c_void
                },
                arg_values.as_ptr() as *mut _,
            );
        };
        result
    }
    pub fn call_args<A>(&mut self, f: impl Into<*mut c_void>, args: A) -> R
    where
        R: Default,
        A: IntoIterator,
        A::Item: IntoFfiArg,
    {
        let mut arg_boxes = Vec::new();
        let arg_ptrs: Vec<*mut c_void> = args
            .into_iter()
            .map(|a| a.into_ffi(&mut arg_boxes))
            .collect();
        self.call(f, &arg_ptrs)
    }
}

#[macro_export]
macro_rules! ffi_call {
    ($cif:expr, $fn:expr $(, $arg:expr)*) => {{
        let mut _arg_boxes: Vec<Box<dyn std::any::Any>> = Vec::new();
        let mut args: Vec<*mut std::ffi::c_void> = Vec::new();

        $(
            // handle each argument and produce a pointer
            let arg_ptr: *mut std::ffi::c_void = {
                #[allow(unused_mut)]
                let mut ptr: *mut std::ffi::c_void = std::ptr::null_mut();

                // try each supported type
                if let Some(s) = (&$arg as &dyn std::any::Any).downcast_ref::<&str>() {
                    let b = Box::new(std::ffi::CString::new(*s).unwrap());
                    let char_ptr = b.as_ptr();
                    _arg_boxes.push(b);

                    let ptr_box = Box::new(char_ptr);
                    ptr = &*ptr_box as *const _ as *mut _;
                    _arg_boxes.push(ptr_box);
                } else if let Some(s) = (&$arg as &dyn std::any::Any).downcast_ref::<String>() {
                    let b = Box::new(std::ffi::CString::new(s.clone()).unwrap());
                    let char_ptr = b.as_ptr();
                    _arg_boxes.push(b);

                    let ptr_box = Box::new(char_ptr);
                    ptr = &*ptr_box as *const _ as *mut _;
                    _arg_boxes.push(ptr_box);
                } else if let Some(i) = (&$arg as &dyn std::any::Any).downcast_ref::<i64>() {
                    let b = Box::new(*i);
                    ptr = &*b as *const _ as *mut _;
                    _arg_boxes.push(b);
                } else if let Some(f) = (&$arg as &dyn std::any::Any).downcast_ref::<f64>() {
                    let b = Box::new(*f);
                    ptr = &*b as *const _ as *mut _;
                    _arg_boxes.push(b);
                } else {
                    // now produce a compile-time error, not ()
                    panic!("Unsupported argument type in ffi_call!");
                }

                ptr
            };

            args.push(arg_ptr);
        )*

        $cif.call($fn, &args)
    }};
}
