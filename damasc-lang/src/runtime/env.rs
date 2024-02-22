use nom::lib::std::collections::HashSet;
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

    pub(crate) fn replace<'x: 'i, 'y: 'x, 'ii, 'ss, 'vv>(
        &mut self,
        identifiers: impl Iterator<Item = (Identifier<'y>, Value<'s, 'v>)>,
    ) {
        for (id, val) in identifiers {
            self.bindings.entry(id.clone()).and_modify(|old| *old = val);
        }
    }

    pub(crate) fn extract<'x, 'y: 'x, 'ii, 'ss, 'vv>(
        &self,
        identifiers: impl Iterator<Item = &'x Identifier<'y>>,
    ) -> Option<Environment<'i, 's, 'v>> {
        let mut env = Environment::new();

        for id in identifiers {
            let current_value = self.bindings.get(id)?;
            env.bindings.insert(id.deep_clone(), current_value.clone());
        }

        Some(env)
    }

    pub(crate) fn extract_except<'x, 'y: 'x, 'ii, 'ss, 'vv>(
        &self,
        identifiers: impl Iterator<Item = &'x Identifier<'y>>,
        exceptions: impl Iterator<Item = &'x Identifier<'y>>,
    ) -> Result<Environment<'i, 's, 'v>, &'x Identifier<'y>> {
        let mut env = Environment::new();
        let skip = exceptions.collect::<HashSet<_>>();

        for id in identifiers {
            if skip.contains(id) {
                continue;
            }

            let Some(current_value) = self.bindings.get(id) else {
                return Err(id);
            };
            env.bindings.insert(id.deep_clone(), current_value.clone());
        }

        Ok(env)
    }

    pub(crate) fn deep_clone<'ix, 'sx, 'vx>(&self) -> Environment<'ix, 'sx, 'vx> {
        Environment {
            bindings: self
                .bindings
                .iter()
                .map(|(k, v)| (k.deep_clone(), v.deep_clone()))
                .collect(),
        }
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
