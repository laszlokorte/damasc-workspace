use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::{map, recognize, value},
    sequence::delimited, error::context,
};

use crate::{literal::Literal, value::ValueType};

use super::io::{ParserResult, ParserInput};

fn literal_null<'v>(input: ParserInput) -> ParserResult<Literal<'v>> {
    context("literal_null", value(Literal::Null, tag("null")))(input)
}

pub(crate) fn literal_string_raw<'v>(input: ParserInput) -> ParserResult<Cow<'v, str>> {
    map(
        delimited(tag("\""), take_until("\""), tag("\"")),
        |s: ParserInput| Cow::Owned(s.fragment().to_owned().to_owned()),
    )(input)
}

fn literal_string<'v>(input: ParserInput) -> ParserResult<Literal<'v>> {
    context("literal_string", map(literal_string_raw, Literal::String))(input)
}

fn literal_bool<'v>(input: ParserInput) -> ParserResult<Literal<'v>> {
    context("literal_bool", alt((
        value(Literal::Boolean(true), tag("true")),
        value(Literal::Boolean(false), tag("false")),
    )))(input)
}

fn literal_number<'v>(input: ParserInput) -> ParserResult<Literal<'v>> {
    context("literal_number", map(recognize(nom::character::complete::i64), |s: ParserInput| {
        Literal::Number(Cow::Owned(s.fragment().to_owned().to_owned()))
    }))(input)
}

pub(crate) fn literal<'v>(input: ParserInput) -> ParserResult<Literal<'v>> {
    context("literal", alt((
        literal_null,
        literal_string,
        literal_bool,
        literal_number,
        literal_type,
    )))(input)
}

pub(crate) fn literal_type_raw(input: ParserInput) -> ParserResult<ValueType> {
    alt((
        value(ValueType::Type, tag("Type")),
        value(ValueType::Null, tag("Null")),
        value(ValueType::Boolean, tag("Boolean")),
        value(ValueType::Integer, tag("Integer")),
        value(ValueType::Array, tag("Array")),
        value(ValueType::Object, tag("Object")),
        value(ValueType::String, tag("String")),
    ))(input)
}

fn literal_type<'v>(input: ParserInput) -> ParserResult<Literal<'v>> {
    context("literal_type", map(literal_type_raw, Literal::Type))(input)
}
