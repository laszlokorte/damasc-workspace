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

use super::io::{ParserResult, ParserInput, ParserError};

fn no_keyword(input: &ParserInput) -> bool {
    !matches!(input.fragment(), &"where" | &"into" | &"limit" | &"with")
}

fn identifier_name<'e, E:ParserError<'e>>(input: ParserInput<'e>) -> ParserResult<ParserInput,E> {
    recognize(alt((
        pair(alpha1, many0_count(alt((alphanumeric1, tag("_"))))),
        pair(tag("_"), many1_count(alt((alphanumeric1, tag("_"))))),
    )))(input)
}

fn identifier_no_keyword<'v,'e, E:ParserError<'e>>(input: ParserInput<'e>) -> ParserResult<Identifier<'v>,E> {
    context("identifier_plain", map(verify(identifier_name, no_keyword), |name| {
        Identifier {
            name: Cow::Owned(name.fragment().to_owned().to_owned()),
        }
    }))(input)
}

fn identifier_raw<'v,'e, E:ParserError<'e>>(input: ParserInput<'e>) -> ParserResult<Identifier<'v>,E> {
    context("identifier_raw", map(preceded(tag("#"), identifier_name), |name: ParserInput| {
        Identifier {
            name: Cow::Owned(name.to_string()),
        }
    }))(input)
}

pub fn identifier<'v,'e, E:ParserError<'e>>(input: ParserInput<'e>) -> ParserResult<Identifier<'v>,E> {
    context("identifier", alt((identifier_raw, identifier_no_keyword)))(input)
}
