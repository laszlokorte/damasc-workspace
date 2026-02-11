use std::borrow::Cow;

use super::{
    identifier::identifier,
    io::{ParserError, ParserInput, ParserResult},
    literal::{literal, literal_string_raw},
    util::ws,
};
use crate::{
    literal::Literal,
    value::{Value, ValueBag},
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{all_consuming, map, opt},
    error::{context, Error},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, separated_pair, terminated},
    Parser,
};

pub fn single_value<'v>(input: &str) -> Option<Value<'_, 'v>> {
    match all_consuming(value_literal::<Error<ParserInput>>).parse(ParserInput::new(input)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

pub fn value_bag<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<'s, ValueBag<'s, 'v>, E> {
    delimited(
        space0,
        map(separated_list1(ws(tag(";")), value_literal), |values| {
            ValueBag::new(values)
        }),
        alt((ws(tag(";")), space0)),
    )
    .parse(input)
}

pub fn value_bag_all_consuming<'v>(input: &str) -> Option<ValueBag<'_, 'v>> {
    match all_consuming(value_bag::<Error<_>>).parse(ParserInput::new(input)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

fn value_array<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<'s, Value<'s, 'v>, E> {
    context(
        "value_array",
        delimited(
            ws(tag("[")),
            terminated(
                map(
                    separated_list0(ws(tag(",")), map(value_literal, Cow::Owned)),
                    Value::Array,
                ),
                opt(ws(tag(","))),
            ),
            ws(tag("]")),
        ),
    )
    .parse(input)
}

fn value_object<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<'s, Value<'s, 'v>, E> {
    context(
        "value_object",
        map(
            delimited(
                ws(tag("{")),
                terminated(
                    separated_list0(
                        ws(ws(tag(","))),
                        map(
                            separated_pair(
                                alt((map(identifier, |i| i.name), literal_string_raw)),
                                ws(tag(":")),
                                value_literal,
                            ),
                            |(p, v)| (p, Cow::Owned(v)),
                        ),
                    ),
                    opt(ws(tag(","))),
                ),
                ws(tag("}")),
            ),
            |props| Value::Object(props.into_iter().collect()),
        ),
    )
    .parse(input)
}

fn value_literal<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<'s, Value<'s, 'v>, E> {
    context(
        "value_literal",
        alt((value_object, value_array, value_literal_atom)),
    )
    .parse(input)
}

fn value_literal_atom<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<'s, Value<'s, 'v>, E> {
    context(
        "value_lteral_atom",
        map(literal, |l| match l {
            Literal::Null => Value::Null,
            Literal::String(s) => Value::String(s),
            Literal::Number(n) => Value::Integer(n.parse().unwrap()),
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Type(t) => Value::Type(t),
        }),
    )
    .parse(input)
}
