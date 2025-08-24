use iced::keyboard::{Key, Modifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Keyboard key combination with modifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombination {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyCombination {
    /// Build combination from an iced [`Key`] and [`Modifiers`]
    pub fn from_event(key: &Key, modifiers: Modifiers) -> Option<Self> {
        let key_str = match key {
            Key::Character(c) => c.to_uppercase(),
            Key::Named(named) => format!("{:?}", named),
            _ => return None,
        };
        Some(Self {
            key: key_str,
            ctrl: modifiers.control(),
            alt: modifiers.alt(),
            shift: modifiers.shift(),
        })
    }

    /// Parse string like "Ctrl+S" into a combination
    pub fn parse(s: &str) -> Option<Self> {
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut key = None;
        for part in s.split('+') {
            let p = part.trim();
            match p.to_lowercase().as_str() {
                "ctrl" | "cmd" => ctrl = true,
                "alt" => alt = true,
                "shift" => shift = true,
                other => key = Some(other.to_uppercase()),
            }
        }
        key.map(|k| Self {
            key: k,
            ctrl,
            alt,
            shift,
        })
    }

    /// Check if given key/modifiers match this combination
    pub fn matches(&self, key: &Key, modifiers: Modifiers) -> bool {
        self.ctrl == modifiers.control()
            && self.alt == modifiers.alt()
            && self.shift == modifiers.shift()
            && match key {
                Key::Character(c) => c.eq_ignore_ascii_case(&self.key),
                Key::Named(named) => self.key.eq_ignore_ascii_case(&format!("{:?}", named)),
                _ => false,
            }
    }
}

impl fmt::Display for KeyCombination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        parts.push(self.key.clone());
        write!(f, "{}", parts.join("+"))
    }
}

/// Context in which hotkeys operate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HotkeyContext {
    Global,
    Diff,
}

/// Manager storing bindings between commands and key combinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyManager {
    #[serde(default)]
    pub global: HashMap<String, KeyCombination>,
    #[serde(default)]
    pub contexts: HashMap<HotkeyContext, HashMap<String, KeyCombination>>,
}

impl HotkeyManager {
    pub fn bind(&mut self, ctx: HotkeyContext, id: String, combo: KeyCombination) {
        match ctx {
            HotkeyContext::Global => {
                self.global.insert(id, combo);
            }
            _ => {
                self.contexts.entry(ctx).or_default().insert(id, combo);
            }
        }
    }

    pub fn binding(&self, ctx: HotkeyContext, id: &str) -> Option<&KeyCombination> {
        match ctx {
            HotkeyContext::Global => self.global.get(id),
            _ => self.contexts.get(&ctx).and_then(|m| m.get(id)),
        }
    }

    /// Resolve a command id for given key and context
    pub fn get_command(
        &self,
        ctx: HotkeyContext,
        key: &Key,
        modifiers: Modifiers,
    ) -> Option<&str> {
        if let Some(map) = self.contexts.get(&ctx) {
            for (id, combo) in map {
                if combo.matches(key, modifiers) {
                    return Some(id.as_str());
                }
            }
        }
        for (id, combo) in &self.global {
            if combo.matches(key, modifiers) {
                return Some(id.as_str());
            }
        }
        None
    }

    /// Return all bindings as strings for uniqueness checks
    pub fn all_bindings(&self) -> Vec<String> {
        let mut list = Vec::new();
        list.extend(self.global.values().map(|c| c.to_string()));
        for m in self.contexts.values() {
            list.extend(m.values().map(|c| c.to_string()));
        }
        list
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        use crate::app::command_palette::COMMANDS;
        let mut hm = Self {
            global: HashMap::new(),
            contexts: HashMap::new(),
        };
        // Built-in defaults
        hm.bind(
            HotkeyContext::Global,
            "create_file".into(),
            KeyCombination::parse("Ctrl+N").unwrap(),
        );
        hm.bind(
            HotkeyContext::Global,
            "save_file".into(),
            KeyCombination::parse("Ctrl+S").unwrap(),
        );
        hm.bind(
            HotkeyContext::Global,
            "rename_file".into(),
            KeyCombination::parse("F2").unwrap(),
        );
        hm.bind(
            HotkeyContext::Global,
            "delete_file".into(),
            KeyCombination::parse("Delete").unwrap(),
        );
        hm.bind(
            HotkeyContext::Diff,
            "next_diff".into(),
            KeyCombination::parse("F8").unwrap(),
        );
        hm.bind(
            HotkeyContext::Diff,
            "prev_diff".into(),
            KeyCombination::parse("F7").unwrap(),
        );
        hm.bind(
            HotkeyContext::Global,
            "toggle_command_palette".into(),
            KeyCombination::parse("Ctrl+Shift+P").unwrap(),
        );
        // Commands from command palette
        for cmd in COMMANDS {
            if let Some(combo) = KeyCombination::parse(cmd.hotkey) {
                hm.bind(HotkeyContext::Global, cmd.id.to_string(), combo);
            }
        }
        hm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_in_contexts() {
        let mut mgr = HotkeyManager::default();
        mgr.bind(
            HotkeyContext::Diff,
            "special".into(),
            KeyCombination::parse("Ctrl+N").unwrap(),
        );
        let key_s = Key::Character("S".into());
        assert_eq!(
            mgr.get_command(HotkeyContext::Global, &key_s, Modifiers::CTRL),
            Some("save_file"),
        );
        let key_n = Key::Character("N".into());
        assert_eq!(
            mgr.get_command(HotkeyContext::Diff, &key_n, Modifiers::CTRL),
            Some("special"),
        );
        assert_eq!(
            mgr.get_command(HotkeyContext::Global, &key_n, Modifiers::CTRL),
            Some("create_file"),
        );
    }
}
