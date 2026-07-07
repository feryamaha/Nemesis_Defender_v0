use nemesis_publisher::sanitize::{extract_target, sanitize_path};

#[test]
fn test_sanitize_home_path() {
    assert_eq!(sanitize_path("/home/fernando/devproj/foo"), "~/devproj/foo");
}

#[test]
fn test_sanitize_users_path() {
    assert_eq!(sanitize_path("/Users/fernando/devproj/foo"), "~/devproj/foo");
}

#[test]
fn test_sanitize_absolute_other() {
    assert_eq!(sanitize_path("/etc/passwd"), "<redacted>");
}

#[test]
fn test_sanitize_relative() {
    assert_eq!(sanitize_path("relative/path"), "relative/path");
}

#[test]
fn test_extract_target_with_separator() {
    let msg = "NEMESIS SEC - COMANDO NAO PERMITIDO \u{00b7} /home/fernando/devproj/foo";
    assert_eq!(extract_target(msg), Some("~/devproj/foo".to_string()));
}

#[test]
fn test_extract_target_without_separator() {
    assert_eq!(extract_target("sem separador"), None);
}
