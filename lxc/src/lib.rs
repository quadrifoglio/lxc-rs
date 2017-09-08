/// Rust library to interact with LXC.

extern crate lxc_sys as lib;

use std::ffi::CStr;

/// Determine the version of LXC currently
/// in use.
pub fn get_version() -> &'static str {
    unsafe {
        CStr::from_ptr(lib::lxc_get_version())
            .to_str().unwrap()
    }
}

#[cfg(test)]
mod tests;
