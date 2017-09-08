/// Tests module.

use super::{Container, Template};

#[test]
fn test_get_version() {
    assert!(super::get_version().len() > 0);
}

#[test]
fn test_create_exists_destroy_container() {
    let ct = Container::create("/var/lib/lxc", "wesh", Template::new("debian"));
    assert!(ct.is_ok());

    let exists = Container::exists("/var/lib/lxc", "wesh");
    assert_eq!(exists, true);

    let ct = ct.unwrap();
    assert!(ct.destroy().is_ok());
}

#[test]
fn test_create_get_destroy_container() {
    let ct = Container::create("/var/lib/lxc", "fromage", Template::new("debian"));
    assert!(ct.is_ok());

    assert!(Container::get("/var/lib/lxc", "fromage").is_ok());

    let ct = ct.unwrap();
    assert_eq!(ct.name, "fromage".to_owned());

    assert!(ct.destroy().is_ok());
}
