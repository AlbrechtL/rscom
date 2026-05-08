use crossterm::event::KeyCode;

use rscom::config::MacroBindingConfig;
use rscom::macros::{MacroSendMode, MacroSet};

#[test]
fn builds_macro_set_and_finds_binding() {
    let cfg = vec![
        MacroBindingConfig {
            key: "F1".to_string(),
            command: "printenv".to_string(),
            description: None,
            char_delay_ms: None,
            echo_timeout_ms: None,
        },
        MacroBindingConfig {
            key: "F2".to_string(),
            command: "saveenv".to_string(),
            description: Some("persist env".to_string()),
            char_delay_ms: None,
            echo_timeout_ms: None,
        },
    ];

    let macros = MacroSet::from_config(&cfg).expect("build macro set");
    let f1 = macros
        .binding_for_keycode(&KeyCode::F(1))
        .expect("find f1 binding");
    assert_eq!(f1.command, "printenv");

    let f3 = macros.binding_for_keycode(&KeyCode::F(3));
    assert!(f3.is_none());
}

#[test]
fn rejects_duplicate_macro_keys() {
    let cfg = vec![
        MacroBindingConfig {
            key: "F4".to_string(),
            command: "foo".to_string(),
            description: None,
            char_delay_ms: None,
            echo_timeout_ms: None,
        },
        MacroBindingConfig {
            key: "f4".to_string(),
            command: "bar".to_string(),
            description: None,
            char_delay_ms: None,
            echo_timeout_ms: None,
        },
    ];

    let err = MacroSet::from_config(&cfg).expect_err("expected duplicate key error");
    assert!(err.to_string().contains("duplicate"));
}

#[test]
fn accepts_newline_only_macro_command() {
    let cfg = vec![MacroBindingConfig {
        key: "F1".to_string(),
        command: "\n".to_string(),
        description: Some("send enter".to_string()),
        char_delay_ms: None,
        echo_timeout_ms: None,
    }];

    let macros = MacroSet::from_config(&cfg).expect("newline command should be valid");
    let f1 = macros
        .binding_for_keycode(&KeyCode::F(1))
        .expect("f1 binding must exist");
    assert_eq!(f1.command, "\n");
}

#[test]
fn default_send_mode_is_immediate() {
    let cfg = vec![MacroBindingConfig {
        key: "F1".to_string(),
        command: "printenv".to_string(),
        description: None,
        char_delay_ms: None,
        echo_timeout_ms: None,
    }];

    let macros = MacroSet::from_config(&cfg).expect("build macro set");
    let f1 = macros.binding_for_keycode(&KeyCode::F(1)).expect("f1");
    assert_eq!(f1.send_mode, MacroSendMode::Immediate);
}

#[test]
fn char_delay_ms_sets_char_delay_mode() {
    let cfg = vec![MacroBindingConfig {
        key: "F2".to_string(),
        command: "boot".to_string(),
        description: None,
        char_delay_ms: Some(50),
        echo_timeout_ms: None,
    }];

    let macros = MacroSet::from_config(&cfg).expect("build macro set");
    let f2 = macros.binding_for_keycode(&KeyCode::F(2)).expect("f2");
    assert_eq!(f2.send_mode, MacroSendMode::CharDelay { ms: 50 });
}

#[test]
fn echo_timeout_ms_sets_echo_sync_mode() {
    let cfg = vec![MacroBindingConfig {
        key: "F3".to_string(),
        command: "saveenv".to_string(),
        description: None,
        char_delay_ms: None,
        echo_timeout_ms: Some(200),
    }];

    let macros = MacroSet::from_config(&cfg).expect("build macro set");
    let f3 = macros.binding_for_keycode(&KeyCode::F(3)).expect("f3");
    assert_eq!(f3.send_mode, MacroSendMode::EchoSync { timeout_ms: 200 });
}

#[test]
fn rejects_both_delay_fields_set() {
    use rscom::config::AppConfig;

    let mut cfg = AppConfig::default();
    cfg.serial.device = "/dev/ttyUSB0".to_string();
    cfg.serial.baud_rate = 115200;
    cfg.macros = vec![MacroBindingConfig {
        key: "F1".to_string(),
        command: "boot".to_string(),
        description: None,
        char_delay_ms: Some(10),
        echo_timeout_ms: Some(200),
    }];

    let err = cfg.validate().expect_err("expected mutual exclusion error");
    assert!(err.to_string().contains("mutually exclusive"));
}
