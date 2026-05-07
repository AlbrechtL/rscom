use std::collections::HashMap;

use crossterm::event::KeyCode;

use crate::config::MacroBindingConfig;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct MacroBinding {
    pub command: String,
    pub description: Option<String>,
}

#[derive(Debug, Default)]
pub struct MacroSet {
    bindings: HashMap<u8, MacroBinding>,
}

impl MacroSet {
    pub fn from_config(items: &[MacroBindingConfig]) -> Result<Self, AppError> {
        let mut bindings = HashMap::new();

        for item in items {
            let key = parse_macro_key(&item.key)?;
            if bindings.contains_key(&key) {
                return Err(AppError::Config(format!(
                    "duplicate macro binding for F{key}"
                )));
            }

            if item.command.is_empty() {
                return Err(AppError::Config(format!(
                    "macro command for {} must not be empty",
                    item.key
                )));
            }

            bindings.insert(
                key,
                MacroBinding {
                    command: item.command.clone(),
                    description: item.description.clone(),
                },
            );
        }

        Ok(Self { bindings })
    }

    pub fn binding_for_keycode(&self, code: &KeyCode) -> Option<&MacroBinding> {
        match code {
            KeyCode::F(n) => self.bindings.get(n),
            _ => None,
        }
    }

    pub fn list(&self) -> impl Iterator<Item = (&u8, &MacroBinding)> {
        self.bindings.iter()
    }
}

pub fn parse_macro_key(raw: &str) -> Result<u8, AppError> {
    let upper = raw.trim().to_ascii_uppercase();
    if !upper.starts_with('F') {
        return Err(AppError::Config(format!(
            "macro key '{raw}' is invalid; expected F1..F12"
        )));
    }

    let idx = upper[1..]
        .parse::<u8>()
        .map_err(|_| AppError::Config(format!("macro key '{raw}' is invalid; expected F1..F12")))?;

    if !(1..=12).contains(&idx) {
        return Err(AppError::Config(format!(
            "macro key '{raw}' is invalid; expected F1..F12"
        )));
    }

    Ok(idx)
}
