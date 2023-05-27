use std::{borrow::Cow, collections::HashSet};

use damasc_lang::{
    identifier::Identifier,
    runtime::{env::Environment, evaluation::Evaluation},
};

use crate::{
    bag::Bag,
    bag_bundle::BagBundle,
    identity::IdentifiedEnvironment,
    iter::BagMultiPredicateIterator,
    join::Join,
    operations::{Deletion, Insertion, Transaction},
};

#[derive(Default, Clone)]
pub struct Controller<'s, 'v> {
    pub storage: BagBundle<'s, 'v>,
}

impl<'s, 'v> Controller<'s, 'v> {
    pub fn query<'x: 's, 'slf: 's>(
        &'slf self,
        join: &'x Join<'s, 'v>,
    ) -> impl Iterator<Item = Transaction<'_, '_>> {
        use itertools::Itertools;

        join.input
            .iter()
            .map(|(source, pred)| {
                let bag = match source {
                    crate::join::JoinSource::Constant(value_bag) => {
                        let adhoc_bag = Bag::from(value_bag);
                        BagMultiPredicateIterator::new(
                            Environment::default(),
                            Identifier::new("?"),
                            pred,
                            Cow::Owned(adhoc_bag),
                        )
                    }
                    crate::join::JoinSource::Named(name) => {
                        if let Some(b) = self.storage.bags.get(name) {
                            BagMultiPredicateIterator::new(
                                Environment::default(),
                                name.clone(),
                                pred,
                                Cow::Borrowed(b),
                            )
                        } else {
                            BagMultiPredicateIterator::empty(
                                Environment::default(),
                                name.clone(),
                                pred,
                            )
                        }
                    }
                };

                bag
            })
            .multi_cartesian_product()
            .filter_map(|x| {
                x.iter()
                    .fold(Some(IdentifiedEnvironment::default()), |acc, e| {
                        let a = acc?;
                        let b = e.as_ref().ok()?;
                        a.combine(b)
                    })
            })
            .filter_map(|e| {
                let mut insertions = HashSet::<Insertion>::default();

                for (sink, expr_set) in &join.output {
                    match sink {
                        crate::join::JoinSink::Print => {
                            continue;
                        }
                        crate::join::JoinSink::Named(sink) => {
                            let ev = Evaluation::new(&e.environment);

                            for expr in &expr_set.expressions {
                                let Ok(val) = ev.eval_expr(expr) else {
                                return None;
                            };

                                insertions.insert(Insertion {
                                    bag_id: sink.clone(),
                                    value: val,
                                });
                            }
                        }
                    }
                }

                Some(Transaction {
                    insertions,
                    deletions: e
                        .used_ids
                        .iter()
                        .map(|id| Deletion {
                            bag_id: id.bag_id.clone(),
                            value_id: id.value_id.clone(),
                        })
                        .collect(),
                    condition: HashSet::new(),
                })
            })
    }
}
