/// Tests module.

use super::{Container, Template};

#[test]
fn get_version() {
    assert!(super::get_version().len() > 0);
}

#[test]
fn create_get_start_freeze_unfreeze_stop_destroy_container() {
    // Create container
    let ct = Container::create("/var/lib/lxc", "fromage", Template::new("debian"));
    assert!(ct.is_ok());

    // Verify it exists
    let exists = Container::exists("/var/lib/lxc", "fromage");
    assert_eq!(exists, true);

    // Verify we can get its informations
    assert!(Container::get("/var/lib/lxc", "fromage").is_ok());

    let ct = ct.unwrap();
    assert_eq!(ct.name, "fromage".to_owned());

    // Verify that it can be started
    assert!(ct.start().is_ok());

    // Verify its state
    assert!(ct.is_running());
    assert_eq!(ct.state(), "RUNNING");

    // Verify that it can be frozen
    assert!(ct.freeze().is_ok());
    assert_eq!(ct.state(), "FROZEN");

    // Verify that it can be unfrozen
    assert!(ct.unfreeze().is_ok());

    // Verify that it can be stopped
    assert!(ct.stop().is_ok());

    // Verify it can be destroyed
    assert!(ct.destroy().is_ok());
}

#[test]
fn create_download_container() {
    // Verify that the download template works
    let template = Template::new("download")
        .option("-d", "alpine")
        .option("-r", "3.6")
        .option("-a", "amd64");

    // Create container
    let ct = Container::create("/var/lib/lxc", "reblochon", template);
    assert!(ct.is_ok());

    // Destroy container
    let ct = ct.unwrap();
    assert!(ct.destroy().is_ok());
}

#[test]
fn create_config_container() {
    // Create a container
    let ct = Container::create("/var/lib/lxc", "calice", Template::new("debian"));
    assert!(ct.is_ok());
    let ct = ct.unwrap();

    // Verify that it has a valid configration file
    let conf_path = ct.get_config_file_name();

    assert!(conf_path.is_ok());
    assert_eq!(conf_path.unwrap().as_str(), "/var/lib/lxc/calice/config");

    // Verify that we can get & set configuration values
    let val = ct.get_config_item("lxc.utsname");
    assert!(val.is_ok());
    assert_eq!(val.unwrap().as_str(), "calice");
    assert!(ct.set_config_item("lxc.utsname", "tamer").is_ok());

    let val = ct.get_config_item("lxc.utsname").unwrap();
    assert_eq!(val.as_str(), "tamer");

    // Destroy it
    assert!(ct.destroy().is_ok());
}

#[test]
fn create_snapshot_restore_container() {
    // Create a container
    let ct = Container::create("/var/lib/lxc", "caribou", Template::new("debian"));
    assert!(ct.is_ok());
    let ct = ct.unwrap();

    // Verify that it can be started and stopped
    assert!(ct.start().is_ok());
    assert!(ct.stop().is_ok());

    // Try taking a snapshot
    let snap = ct.snapshot(None);
    assert!(snap.is_ok());

    // Verify that the listing contains the snapshot
    let snaps = ct.snapshot_list();
    assert!(snaps.is_ok());

    let snaps = snaps.unwrap();
    assert!(snaps.len() > 0);

    // Verify snapshot's information
    let snap = &snaps[0];
    assert_eq!(snap.name.as_str(), "snap0");
    assert!(snap.created.len() > 0);

    // Restore the created snapshot
    assert!(ct.snapshot_restore(snap.name.as_str(), ct.name.as_str()).is_ok());

    // Destroy the snapshot
    assert!(ct.snapshot_destroy(snap.name.as_str()).is_ok());

    // Destroy the container
    assert!(ct.destroy().is_ok());
}
