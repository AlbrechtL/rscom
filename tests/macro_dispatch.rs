use crossterm::event::KeyCode;

use rscom::config::MacroBindingConfig;
use rscom::macros::MacroSet;

#[test]
fn builds_macro_set_and_finds_binding() {
    let cfg = vec![
        MacroBindingConfig {
            key: "F1".to_string(),
            command: "printenv".to_string(),
            description: None,
        },
        MacroBindingConfig {
            key: "F2".to_string(),
            command: "saveenv".to_string(),
            description: Some("persist env".to_string()),
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
        },
        MacroBindingConfig {
            key: "f4".to_string(),
            command: "bar".to_string(),
            description: None,
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
    }];

    let macros = MacroSet::from_config(&cfg).expect("newline command should be valid");
    let f1 = macros
        .binding_for_keycode(&KeyCode::F(1))
        .expect("f1 binding must exist");
    assert_eq!(f1.command, "\n");
}
