use std::collections::HashSet;

use damasc_lang::{identifier::Identifier, value::Value};

use crate::identity::ValueId;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Insertion<'s, 'v> {
    pub bag_id: Identifier<'s>,
    pub value: Value<'s, 'v>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Deletion<'s> {
    pub bag_id: Identifier<'s>,
    pub value_id: ValueId,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ExistenceCondition<'s> {
    bag_id: Identifier<'s>,
    value_id: ValueId,
}

#[derive(Debug)]
pub struct Transaction<'s, 'v> {
    pub insertions: HashSet<Insertion<'s, 'v>>,
    pub deletions: HashSet<Deletion<'s>>,
    pub condition: HashSet<ExistenceCondition<'s>>,
}
