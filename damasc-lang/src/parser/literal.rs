use crate::value_type::ValueType;
use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::{map, recognize, value},
    error::{context, ParseError},
    sequence::delimited,
};

use crate::literal::Literal;

use super::io::{ParserError, ParserInput, ParserResult};

fn literal_null<'v, 'e, E: ParserError<'e>>(
    input: ParserInput<'e>,
) -> ParserResult<Literal<'v>, E> {
    context("literal_null", value(Literal::Null, tag("null")))(input)
}

pub(crate) fn literal_string_raw<'v, 'e, E: ParserError<'e>>(
    input: ParserInput<'e>,
) -> ParserResult<Cow<'v, str>, E> {
    map(
        delimited(tag("\""), take_until("\""), tag("\"")),
        |s: ParserInput| Cow::Owned(s.fragment().to_owned().to_owned()),
    )(input)
}

fn literal_string<'v, 'e, E: ParserError<'e>>(
    input: ParserInput<'e>,
) -> ParserResult<Literal<'v>, E> {
    context("literal_string", map(literal_string_raw, Literal::String))(input)
}

fn literal_bool<'v, 'e, E: ParserError<'e>>(
    input: ParserInput<'e>,
) -> ParserResult<Literal<'v>, E> {
    context(
        "literal_bool",
        alt((
            value(Literal::Boolean(true), tag("true")),
            value(Literal::Boolean(false), tag("false")),
        )),
    )(input)
}

fn literal_number<'v, 'e, E: ParserError<'e>>(
    input: ParserInput<'e>,
) -> ParserResult<Literal<'v>, E> {
    context(
        "literal_number",
        map(
            recognize(nom::character::complete::i64),
            |s: ParserInput| Literal::Number(Cow::Owned(s.fragment().to_owned().to_owned())),
        ),
    )(input)
}

pub(crate) fn literal<'v, 'e, E: ParserError<'e>>(
    input: ParserInput<'e>,
) -> ParserResult<Literal<'v>, E> {
    context(
        "literal",
        alt((
            literal_null,
            literal_string,
            literal_bool,
            literal_number,
            literal_type,
        )),
    )(input)
}

pub(crate) fn literal_type_raw<'s, E: ParseError<ParserInput<'s>>>(
    input: ParserInput<'s>,
) -> ParserResult<ValueType, E> {
    alt((
        value(ValueType::Type, tag("Type")),
        value(ValueType::Null, tag("Null")),
        value(ValueType::Boolean, tag("Boolean")),
        value(ValueType::Integer, tag("Integer")),
        value(ValueType::Array, tag("Array")),
        value(ValueType::Object, tag("Object")),
        value(ValueType::String, tag("String")),
        value(ValueType::Lambda, tag("Lambda")),
    ))(input)
}

fn literal_type<'v, 'e, E: ParserError<'e>>(
    input: ParserInput<'e>,
) -> ParserResult<Literal<'v>, E> {
    context("literal_type", map(literal_type_raw, Literal::Type))(input)
}
