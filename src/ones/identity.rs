use serde_json::Value;
use std::{collections::HashMap, sync::Mutex};

#[derive(Debug, Default)]
pub struct IdentityRegistry {
    aliases: Mutex<HashMap<String, String>>,
}

impl IdentityRegistry {
    pub fn scope(&self, enabled: bool) -> IdentityScope<'_> {
        IdentityScope {
            registry: self,
            enabled,
            names: HashMap::new(),
        }
    }

    fn alias(&self, key: String) -> String {
        let mut aliases = self.aliases.lock().expect("identity registry poisoned");
        if let Some(alias) = aliases.get(&key) {
            return alias.clone();
        }
        let alias = format!("用户-{:03}", aliases.len() + 1);
        aliases.insert(key, alias.clone());
        alias
    }
}

pub struct IdentityScope<'a> {
    registry: &'a IdentityRegistry,
    enabled: bool,
    names: HashMap<String, String>,
}

impl IdentityScope<'_> {
    pub fn person(&mut self, value: &Value) -> Option<String> {
        self.person_parts(
            value.get("uuid").and_then(Value::as_str),
            value.get("name").and_then(Value::as_str),
        )
    }

    pub fn person_parts(&mut self, uuid: Option<&str>, name: Option<&str>) -> Option<String> {
        let name = name.map(str::trim).filter(|value| !value.is_empty());
        if !self.enabled {
            return name.map(ToString::to_string);
        }
        let key = uuid
            .filter(|value| !value.is_empty())
            .map(|value| format!("uuid:{value}"))
            .or_else(|| name.map(|value| format!("name:{}", value.to_lowercase())))?;
        let alias = self.registry.alias(key);
        if let Some(name) = name {
            self.names.insert(name.to_string(), alias.clone());
        }
        Some(alias)
    }

    pub fn redact_text(&self, text: String) -> String {
        if !self.enabled || self.names.is_empty() {
            return text;
        }
        let mut names = self.names.iter().collect::<Vec<_>>();
        names.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
        names
            .into_iter()
            .fold(text, |text, (name, alias)| text.replace(name, alias))
    }
}
