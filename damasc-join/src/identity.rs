use std::collections::HashSet;

use damasc_lang::{value::Value, runtime::env::Environment, identifier::Identifier};


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct ValueId {
    id: u64,
}

impl std::fmt::Display for ValueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
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


#[derive(Debug, Clone, Default)]
pub struct IdentifiedEnvironment<'i, 's, 'v> {
    pub used_ids: HashSet<BagAndValueId<'s>>,
    pub environment: Environment<'i, 's, 'v>,
}

impl<'i, 's, 'v> IdentifiedEnvironment<'i, 's, 'v> {
    pub fn combine(&self, other: &Self) -> Option<Self> {
        // TODO cross bag ids
        // || self.used_ids.intersection(&other.used_ids).count() == 0
        if true {
            let combined_env = self.environment.combine(&other.environment)?;

            Some(IdentifiedEnvironment { 
                used_ids: self.used_ids.union(&other.used_ids).cloned().collect(), 
                environment: combined_env 
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BagAndValueId<'s> {
    pub(crate) bag_id: Identifier<'s>,
    pub(crate) value_id: ValueId,
}