/// Rust library to interact with LXC.

extern crate libc;
extern crate lxc_sys as lib;

use libc::{c_char, c_void, c_int};
use std::ffi::{CStr, CString};

/// Custom error type for this library.
#[derive(Debug)]
pub enum Error {
    ContainerAlreadyExists,
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

pub struct Template {
    name: String,
    options: Option<Vec<String>>
}

impl Template {
    /// Create a new Template object.
    pub fn new<S: Into<String>>(name: S) -> Template {
        Template {
            name: name.into(),
            options: None
        }
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

            libc::free(conts as *mut c_void);
            Ok(vec)
        }
    }

    /// Check wether the LXC container with the specified name is
    /// defined in the provided lxcpath.
    pub fn exists(lxcpath: &str, name: &str) -> bool {
        unsafe {
            let lxcpath = CString::new(lxcpath).unwrap();
            let name = CString::new(name).unwrap();

            let ct = lib::lxc_container_new(name.as_ptr(), lxcpath.as_ptr());
            if ct == 0 as *mut lib::lxc_container {
                return false;
            }

            if (*ct).is_defined.unwrap()(ct) {
                return true;
            }

            false
        }
    }

    /// Create a new LXC container.
    pub fn create(lxcpath: &str, name: &str, template: Template) -> Result<Container> {
        unsafe {
            let lxcpath = CString::new(lxcpath).unwrap();
            let name = CString::new(name).unwrap();

            let ct = lib::lxc_container_new(name.as_ptr(), lxcpath.as_ptr());
            if ct == 0 as *mut lib::lxc_container {
                return Err(Error::Unknown);
            }

            if (*ct).is_defined.unwrap()(ct) {
                return Err(Error::ContainerAlreadyExists);
            }

            let mut template_opts: Vec<*const c_char> = Vec::new();

            if let Some(options) = template.options {
                if options.len() > 0 {
                    for opt in &options {
                        template_opts.push(CString::new(opt.as_str()).unwrap().into_raw() as *const c_char);
                    }

                    template_opts.push(0 as *const c_char);
                }
            }

            let ok = (*ct).create.unwrap()(
                ct,
                template.name.as_ptr() as *const c_char,
                0 as *const c_char,
                0 as *mut lib::bdev_specs,
                lib::LXC_CREATE_QUIET as c_int,
                0 as *const *const c_char
            );

            if !ok {
                return Err(Error::Unknown);
            }

            Ok(Container::from_raw(ct))
        }
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.handle as *mut c_void);
        }
    }
}

#[cfg(test)]
mod tests;
