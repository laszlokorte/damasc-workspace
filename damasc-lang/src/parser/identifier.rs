use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1},
    combinator::{map, recognize, verify},
    multi::{many0_count, many1_count},
    sequence::{pair, preceded}, error::context,
};

use crate::identifier::Identifier;

use super::io::{ParserResult, ParserInput};

fn no_keyword(input: &ParserInput) -> bool {
    !matches!(input.fragment(), &"where" | &"into" | &"limit" | &"with")
}

fn identifier_name(input: ParserInput) -> ParserResult<ParserInput> {
    recognize(alt((
        pair(alpha1, many0_count(alt((alphanumeric1, tag("_"))))),
        pair(tag("_"), many1_count(alt((alphanumeric1, tag("_"))))),
    )))(input)
}

fn identifier_no_keyword<'v>(input: ParserInput) -> ParserResult<Identifier<'v>> {
    context("identifier_plain", map(verify(identifier_name, no_keyword), |name| {
        Identifier {
            name: Cow::Owned(name.fragment().to_owned().to_owned()),
        }
    }))(input)
}

fn identifier_raw<'v>(input: ParserInput) -> ParserResult<Identifier<'v>> {
    context("identifier_raw", map(preceded(tag("#"), identifier_name), |name: ParserInput| {
        Identifier {
            name: Cow::Owned(name.to_string()),
        }
    }))(input)
}

pub(crate) fn identifier<'v>(input: ParserInput) -> ParserResult<Identifier<'v>> {
    context("identifier", alt((identifier_raw, identifier_no_keyword)))(input)
}
