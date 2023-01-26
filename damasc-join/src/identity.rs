use std::collections::HashSet;

use damasc_lang::{value::Value, runtime::{matching::Matcher, env::Environment}};


#[derive(PartialEq, Eq, Debug, Clone,Hash)]
pub struct ValueId {
    id: u64,
}

impl ValueId {
    pub fn new(id: u64) -> Self {
        Self {
            id
        }
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct IdSequence {
    next: u64,
}

impl IdSequence {
    pub(crate) fn next(&mut self) -> ValueId {
        let id = self.next;

        self.next += 1;

        ValueId { id }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IdentifiedValue<'s, 'v> {
    pub(crate) id: ValueId,
    pub(crate) value: Value<'s, 'v>,
}

impl<'s, 'v> IdentifiedValue<'s, 'v> {
    pub fn new(id: ValueId, value: Value<'s, 'v>) -> Self {
        Self {
            id, value
        }
    }
}


#[derive(Debug, Clone)]
pub(crate) struct IdentifiedEnvironment<'i, 's, 'v> {
    pub(crate) used_ids: HashSet<ValueId>,
    pub(crate) environment: Environment<'i, 's, 'v>,
}