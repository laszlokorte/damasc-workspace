use std::collections::BTreeMap;

use crate::{identifier::Identifier, value::Value};

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Environment<'i, 's, 'v> {
    pub bindings: BTreeMap<Identifier<'i>, Value<'s, 'v>>,
}

impl<'i, 's, 'v> Environment<'i, 's, 'v> {
    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    pub fn identifiers(&self) -> std::collections::HashSet<&Identifier> {
        self.bindings.keys().collect()
    }

    pub fn combine(&self, other: &Self) -> Option<Self> {
        let mut bindings = self.bindings.clone();

        for (id, value) in &other.bindings {
            match bindings.insert(id.clone(), value.clone()) {
                Some(ref old) => {
                    if old != value {
                        return None;
                    } else {
                        continue;
                    }
                }
                None => continue,
            }
        }

        Some(Environment { bindings })
    }
}

impl Environment<'_, '_, '_> {
    pub const fn new() -> Self {
        Self {
            bindings: BTreeMap::new(),
        }
    }
}

impl std::fmt::Display for Environment<'_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, v) in &self.bindings {
            writeln!(f, "{id} = {v};")?;
        }
        Ok(())
    }
}

impl Default for Environment<'_, '_, '_> {
    fn default() -> Self {
        Self::new()
    }
}

pub static EMPTY_ENVIRONMENT: Environment<'static, 'static, 'static> = Environment::new();
