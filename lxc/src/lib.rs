/// Rust library to interact with LXC.

extern crate libc;
extern crate lxc_sys as lib;

use libc::{c_char, c_void, c_int};
use std::ffi::{CStr, CString};

/// Custom error type for this library.
#[derive(Debug)]
pub enum Error {
    ContainerDoesNotExists,
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

/// Represents an LXC template script used to build
/// a container's rootfs.
pub struct Template {
    name: String,
    options: Vec<String>
}

impl Template {
    /// Create a new Template object.
    pub fn new<S: Into<String>>(name: S) -> Template {
        Template {
            name: name.into(),
            options: Vec::new()
        }
    }

    /// Add a parameter and its value that will
    /// be passed to the template script.
    pub fn option<S: Into<String>>(mut self, opt: S, value: S) -> Template {
        self.options.push(opt.into());
        self.options.push(value.into());
        self
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

    /// Get an LXC container by its name from the specified
    /// lxcpath.
    pub fn get(lxcpath: &str, name: &str) -> Result<Container> {
        unsafe {
            let lxcpath = CString::new(lxcpath).unwrap();
            let name = CString::new(name).unwrap();

            let ct = lib::lxc_container_new(name.as_ptr(), lxcpath.as_ptr());
            if ct == 0 as *mut lib::lxc_container {
                return Err(Error::Unknown);
            }

            if !(*ct).is_defined.unwrap()(ct) {
                return Err(Error::ContainerDoesNotExists);
            }

            Ok(Container::from_raw(ct))
        }
    }

    /// Create a new LXC container.
    pub fn create(lxcpath: &str, name: &str, template: Template) -> Result<Container> {
        unsafe {
            let lxcpath = CString::new(lxcpath).unwrap();
            let name = CString::new(name).unwrap();
            let template_name = CString::new(template.name.as_str()).unwrap();

            let ct = lib::lxc_container_new(name.as_ptr(), lxcpath.as_ptr());
            if ct == 0 as *mut lib::lxc_container {
                return Err(Error::Unknown);
            }

            if (*ct).is_defined.unwrap()(ct) {
                return Err(Error::ContainerAlreadyExists);
            }

            // Convert the Rust String vector to a vector of CString
            let template_opts = template.options
                .into_iter()
                .map(|s| CString::new(s).unwrap())
                .collect::<Vec<CString>>();

            // Construct the array of C char pointers to be passed
            // to liblxc
            let mut ptr_template_opts = template_opts.iter()
                .map(|s| s.as_ptr() as *const c_char)
                .collect::<Vec<*const c_char>>();

            ptr_template_opts.push(0 as *const c_char);

            let ok = (*ct).create.unwrap()(
                ct,
                template_name.as_ptr(),
                0 as *const c_char,
                0 as *mut lib::bdev_specs,
                lib::LXC_CREATE_QUIET as c_int,
                ptr_template_opts.as_ptr()
            );

            if !ok {
                return Err(Error::Unknown);
            }

            Ok(Container::from_raw(ct))
        }
    }

    /// Get the current configuration file name for the
    /// LXC container.
    pub fn get_config_file_name(&self) -> Result<String> {
        unsafe {
            let ptr = (*self.handle).config_file_name.unwrap()(self.handle);

            if ptr == 0 as *mut c_char {
                return Err(Error::Unknown);
            }

            let name = CString::from_raw(ptr).into_string().unwrap();
            Ok(name)
        }
    }

    /// Retreive the value of a configuration
    /// item of an LXC container.
    pub fn get_config_item(&self, key: &str) -> Result<String> {
        unsafe {
            let key = CString::new(key).unwrap();
            let size = (*self.handle).get_config_item.unwrap()(self.handle, key.as_ptr(), 0 as *mut c_char, 0);

            if size < 0 {
                return Err(Error::Unknown);
            }

            // Allocate a string long enough to hold the returned value
            let mut value = vec![0u8; (size + 1) as usize];

            let ok = (*self.handle).get_config_item.unwrap()(
                self.handle,
                key.as_ptr(),
                value.as_mut_ptr() as *mut c_char,
                size + 1
            );

            // Remove the null byte terminating the returned C string
            value.pop();

            if ok < 0 {
                return Err(Error::Unknown);
            }

            Ok(String::from_utf8(value).unwrap())
        }
    }

    /// Set a key/value configuration option for an
    /// LXC container.
    pub fn set_config_item(&self, key: &str, value: &str) -> Result<()> {
        unsafe {
            let key = CString::new(key).unwrap();
            let value = CString::new(value).unwrap();

            if !(*self.handle).set_config_item.unwrap()(self.handle, key.as_ptr(), value.as_ptr()) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Start the LXC container.
    pub fn start(&self) -> Result<()> {
        unsafe {
            if !(*self.handle).start.unwrap()(self.handle, 0 as c_int, 0 as *const *const c_char) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Freeze a running LXC container.
    pub fn freeze(&self) -> Result<()> {
        unsafe {
            if !(*self.handle).freeze.unwrap()(self.handle) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Thaw a frozen LXC container.
    pub fn unfreeze(&self) -> Result<()> {
        unsafe {
            if !(*self.handle).unfreeze.unwrap()(self.handle) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Request the container to shutdown. Returns an error
    /// if the container failed to shutdown in the
    /// given time.
    pub fn shutdown(&self, timeout: i32) -> Result<()> {
        unsafe {
            if !(*self.handle).shutdown.unwrap()(self.handle, timeout) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Start the LXC container.
    pub fn stop(&self) -> Result<()> {
        unsafe {
            if !(*self.handle).stop.unwrap()(self.handle) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Destroy the LXC container.
    pub fn destroy(self) -> Result<()> {
        unsafe {
            if !(*self.handle).destroy.unwrap()(self.handle) {
                return Err(Error::Unknown);
            }

            Ok(())
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
