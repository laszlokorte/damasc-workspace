use std::borrow::Cow;

use damasc_lang::{value::{Value, ValueBag}, runtime::{env::Environment}};
use damasc_query::predicate::{MultiPredicate};

use crate::{iter::BagMultiPredicateIterator, identity::{IdSequence, IdentifiedValue, ValueId}};

#[derive(Default, Debug, Clone)]
pub struct Bag<'s, 'v> {
    sequence: IdSequence,
    pub(crate) values: Vec<IdentifiedValue<'s, 'v>>,
}

impl<'s, 'v> Bag<'s, 'v> {
    pub fn new() -> Self {
        Self {
            sequence: IdSequence::default(),
            values: Vec::default(),
        }
    }

    pub fn insert(&mut self, value: &Value<'s, 'v>) {
        self.values.push(IdentifiedValue::new(
            self.sequence.next(),
            value.clone(),
        ));
    }

    pub fn remove(&mut self, value_id: ValueId) {
        self.values.retain(|v| v.id != value_id)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

}

impl<'s, 'v:'s> From<ValueBag<'s, 'v>> for Bag<'s, 'v> {
    fn from(value_bag: ValueBag<'s, 'v>) -> Self {
        let mut result = Self::new();
        for v in &value_bag.values {
            result.insert(v);
        }

        result
    }
}

impl<'s, 'v:'s> From<&ValueBag<'s, 'v>> for Bag<'s, 'v> {
    fn from(value_bag: &ValueBag<'s, 'v>) -> Self {
        let mut result = Self::new();
        for v in &value_bag.values {
            result.insert(v);
        }

        result
    }
}

