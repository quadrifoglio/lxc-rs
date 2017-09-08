/// Tests module.

use super::{Container, Template};

#[test]
fn test_get_version() {
    assert!(super::get_version().len() > 0);
}

#[test]
fn test_create_container() {
    let ct = Container::create("/var/lib/lxc", "wesh", Template::new("debian"));
    assert!(ct.is_ok());
}
