use damasc_lang::{value::{Value, ValueBag}, runtime::{env::Environment}};
use damasc_query::predicate::{MultiPredicate};


use crate::{iter::BagMultiPredicateIterator, identity::{IdSequence, IdentifiedValue, ValueId}};

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

    pub(crate) fn query_matchings<'p>(&self, pred: &'p MultiPredicate<'s>) -> BagMultiPredicateIterator<'_, 's, 'v,'p> {
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

