use locos::services::FileService;

#[test]
fn test_file_service_sanitize_key_valid() {
    assert_eq!(
        FileService::sanitize_key("uploads/2026/image.png"),
        Some("uploads/2026/image.png")
    );
    assert_eq!(
        FileService::sanitize_key("documents/contract.pdf"),
        Some("documents/contract.pdf")
    );
}

#[test]
fn test_file_service_sanitize_key_prevents_path_traversal() {
    assert_eq!(FileService::sanitize_key("/etc/passwd"), None);
    assert_eq!(FileService::sanitize_key("uploads/../etc/passwd"), None);
    assert_eq!(FileService::sanitize_key("..\\Windows\\System32"), None);
}
