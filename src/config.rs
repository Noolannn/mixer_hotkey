use std::{error::Error, fmt::Display, io, path::{Path, PathBuf}, str::FromStr};

use serde::{Deserialize, Serialize, ser::SerializeStruct};
use serde_derive::{Deserialize, Serialize};
use windows::Win32::UI::Input::KeyboardAndMouse::{HOT_KEY_MODIFIERS, MOD_CONTROL, MOD_SHIFT};

use crate::utils::{AudioSession, HotKey, HotKeyAction, modifier_from_u32};

#[derive(Debug)]
pub enum LoadError {
    FileNotFound(io::Error),
    DeserializationError(toml::de::Error),
}

impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return match self {
            Self::FileNotFound(e) => Some(e),
            Self::DeserializationError(e) => Some(e),
        }
    }
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(e) => e.fmt(f),
            Self::DeserializationError(e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for LoadError {
    fn from(value: io::Error) -> Self {
        Self::FileNotFound(value)
    }
}

impl From<toml::de::Error> for LoadError {
    fn from(value: toml::de::Error) -> Self {
        Self::DeserializationError(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bindings: Vec<Binding>,
}

impl Config {
    pub fn new() -> Self {
        Self { bindings: vec![] }
    }

    pub fn load_from(path: &Path) -> Result<Self, LoadError> {
        let file_content = std::fs::read_to_string(path)?;
        Ok(toml::from_str::<Config>(&file_content)?)
    }

    pub fn create_hotkeys(&self) -> Vec<HotKey> {
        let mut hotkey_list = vec![];
        for i in 0..self.bindings.len() {
            let binding = &self.bindings[i];
            let hotkey = binding.create_hotkey(i as i32);
            match hotkey {
                Some(h) => hotkey_list.push(h),
                None => continue
            }
        }
        return hotkey_list;
    }

    fn empty_config() -> Self {
        Self { bindings: vec![
            Binding::new(0, 0, "", 0)
        ] }
    }
}

#[derive(Debug, Deserialize)]
pub struct Binding {
    key: u32,
    modifier: u32,
    app: String,
    delta: i32,
}

impl Binding {
    pub fn new(key: u32, modifier: u32, app: &str, delta: i32) -> Self {
        Self { key, modifier, app: app.to_owned(), delta }
    }

    pub fn create_hotkey(&self, id: i32) -> Option<HotKey> {
        let sessions = match AudioSession::get_sessions() {
            Ok(s) => s,
            Err(_err) => return None
        };
        let mut matching_sessions = vec![];
        for s in sessions {
            if s.name.contains(&self.app) {
                matching_sessions.push(s);
            }
        }
        let action = match self.delta {
            0 => HotKeyAction::ToggleMute,
            d => HotKeyAction::ChangeVolume((d as f32)/100.0)
        };
        let hotkey = HotKey::new(id, modifier_from_u32(self.modifier).unwrap(), self.key, matching_sessions, action);
        Some(hotkey)
    }
}

impl Serialize for Binding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut state = serializer.serialize_struct("Binding", 3)?;
        state.serialize_field("app", &self.app)?;
        state.serialize_field("key", &self.key)?;
        state.serialize_field("modifier", &self.modifier)?;
        state.serialize_field("delta", &self.delta)?;
        state.end()
    }
}