use std::borrow::Cow;
use std::collections::HashSet;

use damasc_lang::identifier::Identifier;
use damasc_lang::runtime::evaluation::Evaluation;
use damasc_lang::{runtime::env::Environment, value::Value};
use damasc_query::predicate::MultiPredicate;
use damasc_query::predicate::PredicateError;
use itertools::Permutations;

use crate::identity::BagAndValueId;
use crate::{
    bag::Bag,
    identity::{IdentifiedEnvironment, IdentifiedValue},
};

pub(crate) struct BagMultiPredicateIterator<'i, 's, 'v, 'p> {
    bag_id: Option<Identifier<'s>>,
    env: Environment<'i, 's, 'v>,
    predicate: &'p MultiPredicate<'s>,
    iter: Permutations<std::vec::IntoIter<IdentifiedValue<'s, 'v>>>,
}

impl<'i, 's, 'v, 'p> Clone for BagMultiPredicateIterator<'i, 's, 'v, 'p> {
    fn clone(&self) -> Self {
        Self {
            bag_id: self.bag_id.clone(),
            env: self.env.clone(),
            predicate: self.predicate,
            iter: self.iter.clone(),
        }
    }
}

impl<'i, 's, 'v, 'p> BagMultiPredicateIterator<'i, 's, 'v, 'p> {
    pub fn new(
        env: Environment<'i, 's, 'v>,
        bag_id: Identifier<'s>,
        predicate: &'p MultiPredicate<'s>,
        bag: Cow<'i, Bag<'s, 'v>>,
    ) -> Self {
        use itertools::Itertools;

        Self {
            bag_id: Some(bag_id),
            env,
            iter: bag
                .values
                .clone()
                .into_iter()
                .permutations(predicate.capture.patterns.patterns.len()),
            predicate,
        }
    }

    pub fn new_without_id(
        env: Environment<'i, 's, 'v>,
        predicate: &'p MultiPredicate<'s>,
        bag: Cow<'i, Bag<'s, 'v>>,
    ) -> Self {
        use itertools::Itertools;

        Self {
            bag_id: None,
            env,
            iter: bag
                .values
                .clone()
                .into_iter()
                .permutations(predicate.capture.patterns.patterns.len()),
            predicate,
        }
    }

    pub fn empty(
        env: Environment<'i, 's, 'v>,
        bag_id: Identifier<'s>,
        predicate: &'p MultiPredicate<'s>,
    ) -> Self {
        use itertools::Itertools;

        Self {
            bag_id: Some(bag_id),
            env,
            iter: vec![]
                .into_iter()
                .permutations(predicate.capture.patterns.patterns.len()),
            predicate,
        }
    }
}

impl<'i: 's, 's, 'p> Iterator for BagMultiPredicateIterator<'i, 's, 's, 'p> {
    type Item = Result<IdentifiedEnvironment<'i, 's, 's>, PredicateError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut items = self.iter.next()?;

        loop {
            match (
                &self.bag_id,
                apply_identified(self.predicate, &self.env, items.iter()),
            ) {
                (Some(bag_id), Ok(Some(e))) => {
                    return Some(Ok(IdentifiedEnvironment {
                        used_ids: items
                            .into_iter()
                            .map(|v| BagAndValueId {
                                value_id: v.id,
                                bag_id: bag_id.clone(),
                            })
                            .collect(),
                        environment: e,
                    }))
                }
                (None, Ok(Some(e))) => {
                    return Some(Ok(IdentifiedEnvironment {
                        used_ids: HashSet::new(),
                        environment: e,
                    }))
                }
                (_, Ok(None)) => items = self.iter.next()?,
                (_, Err(e)) => return Some(Err(e)),
            }
        }
    }
}

pub(crate) fn apply_identified<'s: 'x, 'i: 's, 'e, 'x: 'y, 'y>(
    pred: &MultiPredicate<'s>,
    env: &Environment<'i, 's, 's>,
    values: impl Iterator<Item = &'x IdentifiedValue<'s, 's>>,
) -> Result<Option<Environment<'i, 's, 's>>, PredicateError> {
    let env = match pred.capture.apply(env, values.map(|v| &v.value)) {
        Ok(Some(e)) => e,
        Ok(None) => return Ok(None),
        Err(_e) => return Err(PredicateError::PatternError),
    };

    let evaluation = Evaluation::new(&env);

    match evaluation.eval_expr(&pred.guard) {
        Ok(Value::Boolean(b)) => Ok(if b { Some(env) } else { None }),
        Ok(_) => Err(PredicateError::GuardError),
        Err(_) => Err(PredicateError::GuardError),
    }
}
