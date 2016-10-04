extern crate libc;
extern crate itertools;

use libc::{c_int, c_char};
use itertools::Itertools;

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {{
        use std::ffi::CString;
        CString::new($s).unwrap().as_ptr()
    }}
}

pub trait LibcResult<T> {
    /// returns None if the result is empty (-1 if an integer, Null if a pointer)
    /// and Some otherwise
    ///
    /// # Example
    /// if let Some(fd) = libc::creat(fd1, FILE_MODE).to_option() {
    ///     fd
    /// } else {
    ///     panic!("{}", io::Error::last_os_error());
    /// }
    fn to_option(&self) -> Option<T>;
}

impl LibcResult<c_int> for c_int {
    fn to_option(&self) -> Option<c_int> {
        if *self < 0 { None } else { Some(*self) }
    }
}

impl<T> LibcResult<*mut T> for *mut T {
    fn to_option(&self) -> Option<*mut T> {
        if self.is_null() { None } else { Some(*self) }
    }
}

pub trait CArray {
    fn as_char(&self) -> *mut c_char;
}

impl CArray for [c_char] {
    fn as_char(&self) -> *mut c_char {
        self.as_ptr() as *mut c_char
    }
}

pub fn array_to_string(slice: &[i8]) -> String {
    slice.iter().take_while(|&x| *x != 0).map(|&a| a as u8 as char).join("")
}