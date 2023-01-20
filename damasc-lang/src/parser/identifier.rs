use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1},
    combinator::{map, recognize, verify},
    multi::{many0_count, many1_count},
    sequence::{pair, preceded},
    IResult,
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

fn non_keyword_identifier<'v>(input: &str) -> IResult<&str, Identifier<'v>> {
    map(verify(identifier_name, no_keyword), |name: &str| {
        Identifier {
            name: Cow::Owned(name.to_string()),
        }
    })(input)
}

fn raw_identifier<'v>(input: &str) -> IResult<&str, Identifier<'v>> {
    map(preceded(tag("#"), identifier_name), |name: &str| {
        Identifier {
            name: Cow::Owned(name.to_string()),
        }
    })(input)
}

pub(crate) fn identifier<'v>(input: &str) -> IResult<&str, Identifier<'v>> {
    alt((raw_identifier, non_keyword_identifier))(input)
}
