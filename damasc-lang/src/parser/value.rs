use std::borrow::Cow;

use super::{identifier::identifier, literal::literal, util::ws, io::{ParserResult, ParserInput}};
use crate::{
    literal::Literal,
    value::{Value, ValueBag},
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{all_consuming, map, opt},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, separated_pair, terminated}, error::context,
};

pub fn single_value<'v>(input: ParserInput) -> Option<Value<'_, 'v>> {
    match all_consuming(value_literal)(input) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

pub fn value_bag_all_consuming<'v,'s>(input: &'s str) -> Option<ValueBag<'s, 'v>> {
    match all_consuming(delimited(
        space0,
        map(separated_list1(ws(tag(";")), value_literal), |values| {
            ValueBag::new(values)
        }),
        alt((ws(tag(";")), space0)),
    ))(ParserInput::new(input))
    {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

fn value_array<'v,'s>(input: ParserInput<'s>) -> ParserResult<Value<'s, 'v>> {
    context("value_array", delimited(
        ws(tag("[")),
        terminated(
            map(
                separated_list0(ws(tag(",")), map(value_literal, Cow::Owned)),
                Value::Array,
            ),
            opt(ws(tag(","))),
        ),
        ws(tag("]")),
    ))(input)
}

fn value_object<'v,'s>(input: ParserInput<'s>) -> ParserResult<Value<'s, 'v>> {
    context("value_object", map(
        delimited(
            ws(tag("{")),
            terminated(
                separated_list0(
                    ws(ws(tag(","))),
                    map(
                        separated_pair(identifier, tag(":"), value_literal),
                        |(p, v)| (p.name, Cow::Owned(v)),
                    ),
                ),
                opt(ws(tag(","))),
            ),
            ws(tag("}")),
        ),
        |props| Value::Object(props.into_iter().collect()),
    ))(input)
}

fn value_literal<'v,'s>(input: ParserInput<'s>) -> ParserResult<Value<'s, 'v>> {
    context("value_literal", alt((value_object, value_array, value_literal_atom)))(input)
}

fn value_literal_atom<'v>(input: ParserInput) -> ParserResult<Value<'_, 'v>> {
    context("value_lteral_atom", map(literal, |l| match l {
        Literal::Null => Value::Null,
        Literal::String(s) => Value::String(s),
        Literal::Number(n) => Value::Integer(n.parse().unwrap()),
        Literal::Boolean(b) => Value::Boolean(b),
        Literal::Type(t) => Value::Type(t),
    }))(input)
}
