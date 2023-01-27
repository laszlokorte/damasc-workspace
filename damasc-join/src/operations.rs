use std::collections::HashSet;

use damasc_lang::{identifier::Identifier, value::Value};

use crate::identity::ValueId;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Insertion<'s, 'v> {
    bag_id: Identifier<'s>,
    value: Value<'s, 'v>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Deletion<'s> {
    bag_id: Identifier<'s>,
    value_id: ValueId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ExistenceCondition<'s> {
    bag_id: Identifier<'s>,
    value_id: ValueId,
}

struct Transaction<'s,'v> {
    insertions: HashSet<Insertion<'s,'v>>,
    deletions: HashSet<Deletion<'s>>,
    condition: HashSet<ExistenceCondition<'s>>,
}