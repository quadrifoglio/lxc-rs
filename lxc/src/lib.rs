/// Rust library to interact with LXC.

extern crate lxc_sys as lib;

use std::os::raw::c_char;
use std::ffi::{CStr, CString};

/// Custom error type for this library.
#[derive(Debug)]
pub enum Error {
    Unknown
}

/// Custom result type for this library.
pub type Result<T> = std::result::Result<T, Error>;

/// Determine the version of LXC currently
/// in use.
pub fn get_version() -> &'static str {
    unsafe {
        CStr::from_ptr(lib::lxc_get_version())
            .to_str().unwrap()
    }
}

/// LXC Container object representation.
#[derive(Debug)]
pub struct Container {
    handle: *mut lib::lxc_container,

    /// Name of the container.
    pub name: String,
}

impl Container {
    /// Create a Rust container object based on
    /// liblxc's lxc_container C struct.
    fn from_raw(raw: *mut lib::lxc_container) -> Container {
        unsafe {
            let c = *raw;

            Container {
                handle: raw,
                name: CString::from_raw(c.name).into_string().unwrap(),
            }
        }
    }

    /// Get a list of defined LXC containers in the specified
    /// lxcpath.
    pub fn list(lxcpath: &str) -> Result<Vec<Container>> {
        unsafe {
            let mut conts = 0 as *mut *mut lib::lxc_container;

            let count = lib::list_defined_containers(
                lxcpath.as_ptr() as *const c_char,
                0 as *mut *mut *mut c_char,
                &mut conts
            );

            if count < 0 {
                return Err(Error::Unknown);
            }
            else if count == 0 {
                return Ok(Vec::new())
            }

            let mut vec = Vec::with_capacity(count as usize);

            for i in 0..count {
                let elem = *conts.offset(i as isize);
                vec.push(Container::from_raw(elem));
            }

            Ok(vec)
        }
    }
}

#[cfg(test)]
mod tests;
