/// Tests module.

use super::{Container, Template};

#[test]
fn get_version() {
    assert!(super::get_version().len() > 0);
}

#[test]
fn create_exists_destroy_container() {
    let ct = Container::create("/var/lib/lxc", "wesh", Template::new("debian"));
    assert!(ct.is_ok());

    let exists = Container::exists("/var/lib/lxc", "wesh");
    assert_eq!(exists, true);

    let ct = ct.unwrap();
    assert!(ct.destroy().is_ok());
}

#[test]
fn create_get_destroy_container() {
    let ct = Container::create("/var/lib/lxc", "fromage", Template::new("debian"));
    assert!(ct.is_ok());

    assert!(Container::get("/var/lib/lxc", "fromage").is_ok());

    let ct = ct.unwrap();
    assert_eq!(ct.name, "fromage".to_owned());

    assert!(ct.destroy().is_ok());
}

#[test]
fn create_download_destroy_container() {
    let template = Template::new("download")
        .option("-d", "alpine")
        .option("-r", "3.6")
        .option("-a", "amd64");

    let ct = Container::create("/var/lib/lxc", "reblochon", template);
    assert!(ct.is_ok());

    let ct = ct.unwrap();
    assert!(ct.destroy().is_ok());
}

#[test]
fn create_start_stop_destroy_container() {
    let ct = Container::create("/var/lib/lxc", "tabarnacle", Template::new("debian"));
    assert!(ct.is_ok());

    let ct = ct.unwrap();
    assert!(ct.start().is_ok());
    assert!(ct.stop().is_ok());
    assert!(ct.destroy().is_ok());
}

#[test]
fn create_download_start_stop_destroy_container() {
    let template = Template::new("download")
        .option("-d", "centos")
        .option("-r", "7")
        .option("-a", "amd64");

    let ct = Container::create("/var/lib/lxc", "tabarnouche", template);
    assert!(ct.is_ok());

    let ct = ct.unwrap();
    assert!(ct.start().is_ok());
    assert!(ct.stop().is_ok());
    assert!(ct.destroy().is_ok());
}

#[test]
fn create_config_destroy_container() {
    let ct = Container::create("/var/lib/lxc", "calice", Template::new("debian"));
    assert!(ct.is_ok());

    let ct = ct.unwrap();
    let conf_path = ct.get_config_file_name();

    assert!(conf_path.is_ok());
    assert_eq!(conf_path.unwrap().as_str(), "/var/lib/lxc/calice/config");

    assert!(ct.destroy().is_ok());
}
