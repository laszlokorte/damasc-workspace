
use damasc_lang::runtime::env::Environment;

use crate::{bag_bundle::BagBundle, join::Join, bag::Bag, identity::{IdentifiedEnvironment, ValueId, IdentifiedValue}, iter::BagMultiPredicateIterator};

#[derive(Default, Clone)]
pub struct Controller<'s,'v> {
    pub storage: BagBundle<'s,'v>
}


impl<'s,'v> Controller<'s,'v> {

    pub fn query<'x:'s,'slf>(&'slf self, join: &'x Join<'s, 'v>) -> impl Iterator<Item = IdentifiedEnvironment<'_, 's, 'v>> {
        use itertools::Itertools;

        let it = join.input.iter();
        let iter = it.map(|(source, pred)| {
            let bag = match source {
                crate::join::JoinSource::Constant(value_bag) => {
                    //let adhoc_values = value_bag.values.iter().enumerate().map(|(i,v)| 
                   // IdentifiedValue::new(ValueId::new(i as u64),v.clone())).collect();
                   // let adhoc_bag = Bag::new();
                    todo!();
                   //BagMultiPredicateIterator::new(Environment::default(), pred, adhoc_bag)
                },
                crate::join::JoinSource::Named(name) => {
                    if let Some(b) = self.storage.bags.get(name) {
                        b.query_matchings(pred)
                    } else {
                        BagMultiPredicateIterator::empty(Environment::default(), pred)
                    }
                },
            };

            bag
        }).multi_cartesian_product().filter_map(|x| {

            let cc = x.iter().fold(Some(IdentifiedEnvironment::default()), |acc, e| {
                let a = acc?;
                let b = e.as_ref().ok()?;
                a.combine(&b)
            });
            cc
        });

        iter
    }
}