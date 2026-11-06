use citta::config::{read_token, validate_url, write_new_private};
#[test]
fn remote_urls_require_tls_and_disallow_credentials_or_paths() {
    for valid in [
        "https://mac.example.ts.net",
        "http://127.0.0.1:7331",
        "http://[::1]:7331",
    ] {
        assert!(validate_url(valid).is_ok(), "{valid}");
    }
    for invalid in [
        "http://100.64.0.1:7331",
        "http://localhost:7331",
        "https://u:p@example.com",
        "https://example.com/path",
        "https://example.com?token=x",
        "ftp://example.com",
        "https://example.com#fragment",
    ] {
        assert!(validate_url(invalid).is_err(), "{invalid}");
    }
}
#[test]
fn tokens_are_private_and_initialization_never_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("secret");
    write_new_private(&file, b"0123456789012345678901234567890123456789").unwrap();
    assert!(read_token(&file).is_ok());
    assert!(write_new_private(&file, b"replacement").is_err());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read_token(&file).is_err());
    }
}

#[test]
fn identifiers_cannot_name_parent_or_current_directories() {
    for invalid in [".", "..", "../peer", "peer/name", "", "peer name"] {
        assert!(!citta::model::valid_name(invalid));
    }
    for valid in ["desktop", "gpu.worker-1", "job_123"] {
        assert!(citta::model::valid_name(valid));
    }
}
