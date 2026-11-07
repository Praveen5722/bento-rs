use std::path::Path;
#[test]
fn sample_config_has_required_oci_fields() {
    let raw=std::fs::read_to_string("config/config.json").unwrap();
    let v:serde_json::Value=serde_json::from_str(&raw).unwrap();
    assert!(v["ociVersion"].is_string());
    assert!(v["root"]["path"].is_string());
    assert!(v["process"]["args"].is_array());
}
#[test]
fn sample_rootfs_directory_exists() { assert!(Path::new("rootfs").is_dir()); }
