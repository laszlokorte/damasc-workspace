use std::collections::HashMap;

use damasc_lang::{parser::{io::{ParserError, ParserInput, ParserResult}, identifier, value::value_bag, util::ws}, identifier::Identifier};
use nom::{sequence::{pair, delimited}, bytes::complete::tag, multi::fold_many0, combinator::{map, flat_map, all_consuming}, error::Error};

use crate::{bag_bundle::BagBundle, bag::Bag};

pub fn bag_bundle<'v,'s, E:ParserError<'s>>(input: ParserInput<'s>) -> ParserResult<BagBundle<'_,'_>, E>  {
    let (leftover, bags) = fold_many0(
        pair(ws(delimited(tag("#"), identifier::identifier, tag(":"))), value_bag),
        || Ok(HashMap::<Identifier, Bag>::new()),
        |acc, (id, values)| {
            acc.and_then(|mut h| {
                h.try_insert(id, values.into())
                    .map_err(|e| {
                        nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::Count))
                    })?;

                    Ok(h)           
            })
        }
    )(input)?;
    
    bags.map(|bags| (leftover, BagBundle { bags }))
}

pub fn bag_bundle_all_consuming(bundle_string: &str) -> Option<BagBundle<'_,'_>> {
    match all_consuming(bag_bundle::<Error<ParserInput>>)(ParserInput::new(bundle_string)) {
        Ok((_,r)) => Some(r),
        Err(_) => None,
    }
}