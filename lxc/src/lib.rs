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

/// Represents an LXC container snapshot.
pub struct Snapshot {
    handle: lib::lxc_snapshot,

    /// Name of the snapshot.
    pub name: String,

    /// Time the snapshot was created at.
    pub created: String
}

impl Snapshot {
    /// Create a Rust Snapshot object based on a
    /// liblxc lxc_snapshot struct.
    fn from_raw(raw: lib::lxc_snapshot) -> Snapshot {
        unsafe {
            Snapshot {
                handle: raw,
                name: CStr::from_ptr(raw.name as *const c_char).to_str().unwrap().to_string(),
                created: CStr::from_ptr(raw.timestamp as *const c_char).to_str().unwrap().to_string(),
            }
        }
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe {
            self.handle.free.unwrap()(&mut self.handle);
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
                name: CStr::from_ptr(c.name as *const c_char).to_str().unwrap().to_owned()
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

    /// Retrieve a list of config item keys given a key prefix.
    pub fn get_keys(&self, key_prefix: &str) -> Result<Vec<String>> {
        unsafe {
            let key_prefix = CString::new(key_prefix).unwrap();
            let length = (*self.handle).get_keys.unwrap()(self.handle, key_prefix.as_ptr(), 0 as *mut c_char, 0);
            println!("pute {}", length);

            if length < 0 {
                return Err(Error::Unknown);
            }
            else if length == 0 {
                return Ok(Vec::new());
            }

            let mut s = vec![0u8; length as usize];

            let ok = (*self.handle).get_keys.unwrap()(self.handle, key_prefix.as_ptr(), s.as_mut_ptr() as *mut c_char, length);
            if ok < 0 {
                return Err(Error::Unknown);
            }

            let s = String::from_utf8(s).unwrap();
            println!("{}", s);

            Ok(Vec::new())
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

    /// Clear the container's in-memory configuration.
    pub fn clear_config(&self) -> Result<()> {
        unsafe {
            (*self.handle).clear_config.unwrap()(self.handle);
            Ok(())
        }
    }

    /// Clear a specific container configuration item.
    pub fn clear_config_item(&self, key: &str) -> Result<()> {
        unsafe {
            let key = CString::new(key).unwrap();

            if !(*self.handle).clear_config_item.unwrap()(self.handle, key.as_ptr()) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Save the container configuration to a file.
    pub fn save_config(&self, file_path: &str) -> Result<()> {
        unsafe {
            let file_path = CString::new(file_path).unwrap();

            if !(*self.handle).save_config.unwrap()(self.handle, file_path.as_ptr()) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Change wether the container wants to run disconnected
    /// from the terminal.
    pub fn want_daemonize(&self, want_daemonize: bool) -> Result<()> {
        unsafe {
            if !(*self.handle).want_daemonize.unwrap()(self.handle, want_daemonize) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Change wether the container wishes all the file descriptors
    /// to be closed on startup.
    pub fn want_close_all_fds(&self, want_close_all_fds: bool) -> Result<()> {
        unsafe {
            if !(*self.handle).want_close_all_fds.unwrap()(self.handle, want_close_all_fds) {
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

    /// Start the LXC container.
    pub fn stop(&self) -> Result<()> {
        unsafe {
            if !(*self.handle).stop.unwrap()(self.handle) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Check wether a container is running or not.
    pub fn is_running(&self) -> bool {
        unsafe {
            (*self.handle).is_running.unwrap()(self.handle)
        }
    }

    /// Determine the state of a container. Returns an upper-case
    /// word representing the state.
    pub fn state(&self) -> &'static str {
        unsafe {
            let s = (*self.handle).state.unwrap()(self.handle);
            CStr::from_ptr(s).to_str().unwrap()
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

    /// Create a LXC container snapshot with the given path
    /// to the snapshot's comment file. Returns the
    /// zero-based snapshot number.
    pub fn snapshot(&self, comment_file: Option<&str>) -> Result<u32> {
        unsafe {
            let num: i32;

            if let Some(comment_file) = comment_file {
                let comment_file = CString::new(comment_file).unwrap();
                num = (*self.handle).snapshot.unwrap()(self.handle, comment_file.as_ptr());
            }
            else {
                num = (*self.handle).snapshot.unwrap()(self.handle, 0 as *const c_char);
            }

            if num < 0 {
                return Err(Error::Unknown);
            }

            Ok(num as u32)
        }
    }

    /// Obtain a list of container snapshot.
    pub fn snapshot_list(&self) -> Result<Vec<Snapshot>> {
        unsafe {
            let mut ptr = 0 as *mut lib::lxc_snapshot;
            let count = (*self.handle).snapshot_list.unwrap()(self.handle, &mut ptr);

            if count < 0 {
                return Err(Error::Unknown);
            }

            let count = count as usize;
            let vec: Vec<lib::lxc_snapshot> = Vec::from_raw_parts(ptr, count, count);

            let vec = vec.into_iter()
                .map(|s| Snapshot::from_raw(s))
                .collect::<Vec<Snapshot>>();

            Ok(vec)
        }
    }

    /// Restore the specified snapshot as a new container with the
    /// given name. If the given name if identical to the original
    /// container's name, it will be reaplced.
    pub fn snapshot_restore(&self, snap_name: &str, container_name: &str) -> Result<()> {
        unsafe {
            let snap_name = CString::new(snap_name).unwrap();
            let container_name = CString::new(container_name).unwrap();

            if !(*self.handle).snapshot_restore.unwrap()(self.handle, snap_name.as_ptr(), container_name.as_ptr()) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Destroy the specified snapshot.
    pub fn snapshot_destroy(&self, snap_name: &str) -> Result<()> {
        unsafe {
            let snap_name = CString::new(snap_name).unwrap();

            if !(*self.handle).snapshot_destroy.unwrap()(self.handle, snap_name.as_ptr()) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Destroy all the container's snapshots.
    pub fn snapshot_destroy_all(&self) -> Result<()> {
        unsafe {
            if !(*self.handle).snapshot_destroy_all.unwrap()(self.handle) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Checkpoint an LXC container. Dump the checkpoint files in the
    /// specified directory. There is the possibility to stop the
    /// container after the ckeckpoint is done.
    pub fn checkpoint(&self, directory: &str, stop: bool, verbose: bool) -> Result<()> {
        unsafe {
            let directory = CString::new(directory).unwrap();

            if !(*self.handle).checkpoint.unwrap()(self.handle, directory.as_ptr() as *mut c_char, stop, verbose) {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    /// Restore a container from a checkpoint previously dumped into
    /// the specified directory.
    pub fn restore(&self, directory: &str, verbose: bool) -> Result<()> {
        unsafe {
            let directory = CString::new(directory).unwrap();

            if !(*self.handle).restore.unwrap()(self.handle, directory.as_ptr() as *mut c_char, verbose) {
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

    /// Destroy the LXC container and all its snapshots.
    pub fn destroy_with_snapshots(self) -> Result<()> {
        unsafe {
            if !(*self.handle).destroy_with_snapshots.unwrap()(self.handle) {
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
            lib::lxc_container_put(self.handle);
        }
    }
}

#[cfg(test)]
mod tests;
