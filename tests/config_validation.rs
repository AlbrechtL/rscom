use std::io::Write;

use rscom::config::AppConfig;

#[test]
fn parses_minimal_valid_config() {
    let mut file = tempfile::NamedTempFile::new().expect("create temp file");
    writeln!(
        file,
        "serial:\n  device: /dev/ttyUSB0\n  baud_rate: 115200\n"
    )
    .expect("write config");

    let cfg = AppConfig::load_from_path(file.path()).expect("parse config");
    cfg.validate().expect("validate config");
}

#[test]
fn rejects_invalid_stop_bits() {
    let mut file = tempfile::NamedTempFile::new().expect("create temp file");
    writeln!(
        file,
        "serial:\n  device: /dev/ttyUSB0\n  baud_rate: 115200\n  stop_bits: 3\n"
    )
    .expect("write config");

    let cfg = AppConfig::load_from_path(file.path()).expect("parse config");
    let err = cfg.validate().expect_err("expected validation error");
    assert!(
        err.to_string().contains("stop_bits"),
        "unexpected error: {err}"
    );
}
