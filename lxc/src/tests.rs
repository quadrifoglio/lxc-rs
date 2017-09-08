/// Tests module.

#[test]
fn test_get_version() {
    assert!(super::get_version().len() > 0);
}

#[test]
fn test_list_containers() {
    let containers = super::Container::list("/var/lib/lxc");
    assert!(containers.is_ok());
}
