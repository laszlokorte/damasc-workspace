use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1},
    combinator::{map, recognize, verify},
    multi::{many0_count, many1_count},
    sequence::{pair, preceded},
    IResult, error::context,
};

use crate::identifier::Identifier;

fn no_keyword(input: &str) -> bool {
    !matches!(input, "where" | "into" | "limit" | "with")
}

fn identifier_name(input: &str) -> IResult<&str, &str> {
    recognize(alt((
        pair(alpha1, many0_count(alt((alphanumeric1, tag("_"))))),
        pair(tag("_"), many1_count(alt((alphanumeric1, tag("_"))))),
    )))(input)
}

fn identifier_no_keyword<'v>(input: &str) -> IResult<&str, Identifier<'v>> {
    context("identifier_plain", map(verify(identifier_name, no_keyword), |name: &str| {
        Identifier {
            name: Cow::Owned(name.to_string()),
        }
    }))(input)
}

fn identifier_raw<'v>(input: &str) -> IResult<&str, Identifier<'v>> {
    context("identifier_raw", map(preceded(tag("#"), identifier_name), |name: &str| {
        Identifier {
            name: Cow::Owned(name.to_string()),
        }
    }))(input)
}

pub(crate) fn identifier<'v>(input: &str) -> IResult<&str, Identifier<'v>> {
    context("identifier", alt((identifier_raw, identifier_no_keyword)))(input)
}
