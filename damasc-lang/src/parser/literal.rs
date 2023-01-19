use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::{map, recognize, value},
    sequence::delimited,
    IResult,
};

use crate::{literal::Literal, value::ValueType};

fn literal_null<'v>(input: &str) -> IResult<&str, Literal<'v>> {
    value(Literal::Null, tag("null"))(input)
}

pub(crate) fn literal_string_raw<'v>(input: &str) -> IResult<&str, Cow<'v, str>> {
    map(
        delimited(tag("\""), take_until("\""), tag("\"")),
        |s: &str| Cow::Owned(s.to_string()),
    )(input)
}

fn literal_string<'v>(input: &str) -> IResult<&str, Literal<'v>> {
    map(literal_string_raw, Literal::String)(input)
}

fn literal_bool<'v>(input: &str) -> IResult<&str, Literal<'v>> {
    alt((
        value(Literal::Boolean(true), tag("true")),
        value(Literal::Boolean(false), tag("false")),
    ))(input)
}

fn literal_number<'v>(input: &str) -> IResult<&str, Literal<'v>> {
    map(recognize(nom::character::complete::i64), |s: &str| {
        Literal::Number(Cow::Owned(s.to_owned()))
    })(input)
}

pub(crate) fn literal<'v>(input: &str) -> IResult<&str, Literal<'v>> {
    alt((
        literal_null,
        literal_string,
        literal_bool,
        literal_number,
        literal_type,
    ))(input)
}

pub(crate) fn literal_type_raw(input: &str) -> IResult<&str, ValueType> {
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

fn literal_type<'v>(input: &str) -> IResult<&str, Literal<'v>> {
    map(literal_type_raw, Literal::Type)(input)
}
