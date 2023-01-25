use damasc_lang::{value::{Value, ValueBag}, runtime::{env::Environment}};
use damasc_query::predicate::{MultiPredicate};


use crate::iter::BagMultiPredicateIterator;

#[derive(PartialEq, Eq, Debug, Clone)]
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
struct IdSequence {
    next: u64,
}
impl IdSequence {
    fn next(&mut self) -> ValueId {
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

#[derive(Default, Debug, Clone)]
pub struct Bag<'s, 'v> {
    sequence: IdSequence,
    pub(crate) values: Vec<IdentifiedValue<'s, 'v>>,
}

impl<'s, 'v> Bag<'s, 'v> {
    fn new() -> Self {
        Self {
            sequence: IdSequence::default(),
            values: Vec::default(),
        }
    }

    pub fn insert(&mut self, value: &Value<'s, 'v>) {
        self.values.push(IdentifiedValue {
            id: self.sequence.next(),
            value: value.clone(),
        })
    }

    pub fn remove(&mut self, value_id: ValueId) {
        self.values.retain(|v| v.id != value_id)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn query<'x:'s,'y,'p:'s>(&'x self, pred: &'p MultiPredicate<'s>) -> BagMultiPredicateIterator {
        BagMultiPredicateIterator::new(Environment::default(), pred, self)
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

