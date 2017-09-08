/// Tests module.

#[test]
fn test_get_version() {
    assert!(super::get_version().len() > 0);
}
