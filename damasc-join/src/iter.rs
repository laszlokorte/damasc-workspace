use std::collections::HashSet;

use damasc_lang::{runtime::{env::Environment, evaluation::Evaluation}, value::{Value, ValueBag}};
use damasc_query::predicate::{MultiPredicate, PredicateError};
use itertools::Permutations;

use crate::{bag::Bag, identity::{IdentifiedValue, IdentifiedEnvironment}};

pub(crate)  struct BagMultiPredicateIterator<'i, 's, 'v,'p> {
    env: Environment<'i, 's, 'v>,
    predicate: &'p MultiPredicate<'s>,
    iter: Permutations<core::slice::Iter<'i, IdentifiedValue<'s,'v>>>,
}

impl<'i, 's, 'v,'p> Clone for BagMultiPredicateIterator<'i, 's, 'v,'p>
{
    fn clone(&self) -> Self {
        Self {
            env: self.env.clone(),
            predicate: self.predicate,
            iter: self.iter.clone(),
        }
    }
}

impl<'i, 's, 'v,'p> BagMultiPredicateIterator<'i, 's, 'v,'p>
{
    pub fn new(env: Environment<'i, 's, 'v>, predicate: &'p MultiPredicate<'s>, bag: &'i Bag<'s,'v>) -> Self {
        use itertools::Itertools;

        Self {
            env,
            iter: bag.values.iter().permutations(predicate.capture.patterns.patterns.len()),
            predicate,
        }
    }

    pub fn empty(env: Environment<'i, 's, 'v>, predicate: &'p MultiPredicate<'s>) -> Self {
        use itertools::Itertools;

        Self {
            env,
            iter: [].iter().permutations(predicate.capture.patterns.patterns.len()),
            predicate,
        }
    }

}

impl<'i, 's: 'v, 'v,'p> Iterator
    for BagMultiPredicateIterator<'i, 's, 'v,'p>
{
    type Item = Result<IdentifiedEnvironment<'i, 's, 'v>, PredicateError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(items) = self.iter.next() else {
            return None;
        };

        match apply_identified(self.predicate, &self.env, items.iter()) {
            Ok(Some(e)) => Some(Ok(IdentifiedEnvironment { 
                used_ids: items.into_iter().map(|v|v.id.clone()).collect(), 
                environment: e 
            })),
            Ok(None) => self.next(),
            Err(e) => Some(Err(e)),
        }
    }
}


pub(crate) fn apply_identified<'s, 'v: 'x, 'i, 'e, 'x:'y, 'y>(
    pred: &MultiPredicate<'s>,
    env: &Environment<'i, 's, 'v>,
    values: impl Iterator<Item = &'y &'x IdentifiedValue<'s, 'v>>,
) -> Result<Option<Environment<'i, 's, 'v>>, PredicateError> {
    let env = match pred.capture.apply(env, values.map(|v| &v.value)) {
        Ok(Some(e)) => e,
        Ok(None) => return Ok(None),
        Err(_e) => return Err(PredicateError::PatternError),
    };

    let evaluation = Evaluation::new(&env);

    match evaluation.eval_expr(&pred.guard) {
        Ok(Value::Boolean(b)) => Ok(if b {Some(env)}else{None}),
        Ok(_) => Err(PredicateError::GuardError),
        Err(_) => Err(PredicateError::GuardError),
    }
}
