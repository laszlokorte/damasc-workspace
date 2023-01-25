use damasc_lang::{runtime::{env::Environment, evaluation::Evaluation}, value::Value};
use damasc_query::predicate::{MultiPredicate, PredicateError};
use itertools::Permutations;

use crate::bag::{IdentifiedValue, Bag};

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

        let x = bag.values.iter();

        Self {
            env,
            iter: bag.values.iter().permutations(predicate.capture.patterns.patterns.len()),
            predicate,
        }
    }
}

impl<'i, 's: 'v, 'v,'p> Iterator
    for BagMultiPredicateIterator<'i, 's, 'v,'p>
{
    type Item = Result<Vec<&'i IdentifiedValue<'s, 'v>>, PredicateError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(items) = self.iter.next() else {
            return None;
        };

        match apply_identified(&self.predicate, &self.env, items.iter()) {
            Ok(true) => Some(Ok(items)),
            Ok(false) => self.next(),
            Err(e) => Some(Err(e)),
        }
    }
}


pub(crate) fn apply_identified<'s, 'v: 'x, 'i, 'e, 'x:'y, 'y>(
    pred: &MultiPredicate<'s>,
    env: &Environment<'i, 's, 'v>,
    values: impl Iterator<Item = &'y &'x IdentifiedValue<'s, 'v>>,
) -> Result<bool, PredicateError> {
    let env = match pred.capture.apply(env, values.map(|v| &v.value)) {
        Ok(Some(e)) => e,
        Ok(None) => return Ok(false),
        Err(_e) => return Err(PredicateError::PatternError),
    };

    let evaluation = Evaluation::new(&env);

    match evaluation.eval_expr(&pred.guard) {
        Ok(Value::Boolean(b)) => Ok(b),
        Ok(_) => Err(PredicateError::GuardError),
        Err(_) => Err(PredicateError::GuardError),
    }
}
