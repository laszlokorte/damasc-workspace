use std::collections::BTreeMap;

use crate::{identifier::Identifier, value::Value};

#[derive(Clone, Debug)]
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
}

impl Environment<'_, '_, '_> {
    pub const fn new() -> Self {
        Self {
            bindings: BTreeMap::new(),
        }
    }
}

impl Default for Environment<'_, '_, '_> {
    fn default() -> Self {
        Self::new()
    }
}
