/// Tests module.

use super::{Container, Template};

const LXC_PATH: &'static str = "/var/lib/lxc";

#[test]
fn version() {
    assert!(super::version().len() > 0);
}

#[test]
fn create_get_start_freeze_unfreeze_stop_destroy_container() {
    // Create container
    Container::create(LXC_PATH, "fromage", Template::new("debian")).unwrap();

    // Verify it exists
    assert!(Container::exists(LXC_PATH, "fromage"));

    // Verify we can get its informations
    let ct = Container::get(LXC_PATH, "fromage").unwrap();
    assert_eq!(ct.name.as_str(), "fromage");

    // Verify that it can be started
    ct.start().unwrap();

    // Verify its state
    assert!(ct.is_running());
    assert_eq!(ct.state(), "RUNNING");

    // Verify that it can be frozen
    ct.freeze().unwrap();
    assert_eq!(ct.state(), "FROZEN");

    // Verify that it can be unfrozen
    ct.unfreeze().unwrap();

    // Verify that it can be stopped
    ct.stop().unwrap();

    // Verify it can be destroyed
    ct.destroy().unwrap();
}

#[test]
fn create_download_container() {
    // Verify that the download template works
    let template = Template::new("download")
        .option("-d", "alpine")
        .option("-r", "3.6")
        .option("-a", "amd64");

    // Create container
    let ct = Container::create(LXC_PATH, "reblochon", template).unwrap();

    // Destroy container
    ct.destroy().unwrap();
}

#[test]
fn create_config_container() {
    // Create a container
    let ct = Container::create(LXC_PATH, "calice", Template::new("debian")).unwrap();

    // Verify that it has a valid configration file
    let conf_path = ct.get_config_file_name().unwrap();
    assert_eq!(conf_path.as_str(), "/var/lib/lxc/calice/config");

    // Verify that we can get & set configuration values
    let val = ct.get_config_item("lxc.utsname").unwrap();
    assert_eq!(val.as_str(), "calice");

    ct.set_config_item("lxc.utsname", "tamer").unwrap();

    let val = ct.get_config_item("lxc.utsname").unwrap();
    assert_eq!(val.as_str(), "tamer");

    // Destroy it
    ct.destroy().unwrap();
}

#[test]
fn create_snapshot_restore_container() {
    // Create a container
    let ct = Container::create(LXC_PATH, "caribou", Template::new("debian")).unwrap();

    // Verify that it can be started and stopped
    ct.start().unwrap();
    ct.stop().unwrap();

    // Try taking a snapshot
    ct.snapshot(None).unwrap();

    // Verify that the listing contains the snapshot
    let snaps = ct.snapshot_list().unwrap();
    assert!(snaps.len() > 0);

    // Verify snapshot's information
    let snap = &snaps[0];
    assert_eq!(snap.name.as_str(), "snap0");
    assert!(snap.created.len() > 0);

    // Restore the created snapshot
    ct.snapshot_restore(snap.name.as_str(), ct.name.as_str()).unwrap();

    // Destroy the snapshot
    ct.snapshot_destroy(snap.name.as_str()).unwrap();

    // Destroy the container
    ct.destroy().unwrap();
}
