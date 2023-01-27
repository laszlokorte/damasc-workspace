
use std::borrow::Cow;

use damasc_lang::{runtime::env::Environment, identifier::Identifier};

use crate::{bag_bundle::BagBundle, join::Join, bag::Bag, identity::IdentifiedEnvironment, iter::BagMultiPredicateIterator};

#[derive(Default, Clone)]
pub struct Controller<'s,'v> {
    pub storage: BagBundle<'s,'v>
}


impl<'s,'v> Controller<'s,'v> {

    pub fn query<'x:'s,'slf>(&'slf self, join: &'x Join<'s, 'v>) -> impl Iterator<Item = IdentifiedEnvironment<'_, 's, 'v>> {
        use itertools::Itertools;

        join.input.iter().map(|(source, pred)| {
            let bag = match source {
                crate::join::JoinSource::Constant(value_bag) => {
                   let adhoc_bag = Bag::from(value_bag);
                   BagMultiPredicateIterator::new(Environment::default(), Identifier::new("?"), pred, Cow::Owned(adhoc_bag))
                },
                crate::join::JoinSource::Named(name) => {
                    if let Some(b) = self.storage.bags.get(name) {
                        BagMultiPredicateIterator::new(Environment::default(), name.clone(), pred, Cow::Borrowed(b))
                    } else {
                        BagMultiPredicateIterator::empty(Environment::default(), name.clone(), pred)
                    }
                },
            };

            bag
        }).multi_cartesian_product().filter_map(|x| {

            let cc = x.iter().fold(Some(IdentifiedEnvironment::default()), |acc, e| {
                let a = acc?;
                let b = e.as_ref().ok()?;
                a.combine(b)
            });
            cc
        })
    }
}