
use crate::{bag_bundle::BagBundle, join::Join, bag::Bag};

#[derive(Default, Clone)]
struct Controller<'s,'v> {
    storage: BagBundle<'s,'v>
}

impl<'s,'v> Controller<'s,'v> {

    fn query<'x:'s>(&self, join: &'x Join<'s, 'v>) {
        use itertools::Itertools;
        let empty_bag = Bag::default();

        let it = join.input.iter();
        let iter = it.map(|(source, pred)| {
            let bag = match source {
                crate::join::JoinSource::Constant(values) => todo!(),
                crate::join::JoinSource::Named(name) => self.storage.bags.get(name).unwrap_or(&empty_bag),
            };

            bag.query_matchings(pred)
        }).multi_cartesian_product().map(|x| {
            let cc = x.into_iter().collect::<Result<Vec<_>,_>>();
        });

    }
}