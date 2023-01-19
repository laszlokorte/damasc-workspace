use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map, opt, value},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};

use crate::syntax::{
    expression::PropertyKey,
    pattern::{ArrayPatternItem, ObjectPropertyPattern, Pattern, PropertyPattern, Rest},
};

use super::{
    expression::expression,
    identifier::identifier,
    literal::{literal, literal_type_raw},
    util::ws,
};

fn pattern_discard<'v>(input: &str) -> IResult<&str, Pattern<'v>> {
    value(Pattern::Discard, tag("_"))(input)
}

fn pattern_typed_discard<'v>(input: &str) -> IResult<&str, Pattern<'v>> {
    map(
        preceded(ws(tag("_ is ")), literal_type_raw),
        Pattern::TypedDiscard,
    )(input)
}

fn pattern_identifier<'v>(input: &str) -> IResult<&str, Pattern<'v>> {
    map(identifier, Pattern::Identifier)(input)
}

fn pattern_typed_identifier<'v>(input: &str) -> IResult<&str, Pattern<'v>> {
    map(
        separated_pair(identifier, tag(" is "), literal_type_raw),
        |(i, t)| Pattern::TypedIdentifier(i, t),
    )(input)
}

fn object_prop_pattern<'v>(input: &str) -> IResult<&str, ObjectPropertyPattern<'v>> {
    alt((
        map(
            separated_pair(
                delimited(ws(tag("[")), expression, ws(tag("]"))),
                ws(tag(":")),
                pattern,
            ),
            |(prop, value)| {
                ObjectPropertyPattern::Match(PropertyPattern {
                    key: PropertyKey::Expression(prop),
                    value,
                })
            },
        ),
        map(
            separated_pair(identifier, ws(tag(":")), pattern),
            |(prop, value)| {
                ObjectPropertyPattern::Match(PropertyPattern {
                    key: PropertyKey::Identifier(prop),
                    value,
                })
            },
        ),
        map(identifier, ObjectPropertyPattern::Single),
    ))(input)
}

fn pattern_object<'v>(input: &str) -> IResult<&str, Pattern<'v>> {
    delimited(
        ws(tag("{")),
        alt((
            map(pattern_rest, |r| Pattern::Object(vec![], r)),
            map(
                tuple((
                    separated_list0(ws(ws(tag(","))), object_prop_pattern),
                    opt(preceded(ws(tag(",")), opt(pattern_rest))),
                )),
                |(props, rest)| Pattern::Object(props, rest.flatten().unwrap_or(Rest::Exact)),
            ),
        )),
        ws(tag("}")),
    )(input)
}

fn pattern_rest<'v>(input: &str) -> IResult<&str, Rest<'v>> {
    alt((
        map(preceded(ws(tag("...")), pattern), |r| {
            Rest::Collect(Box::new(r))
        }),
        value(Rest::Discard, ws(tag("..."))),
    ))(input)
}

fn pattern_array<'v>(input: &str) -> IResult<&str, Pattern<'v>> {
    delimited(
        ws(tag("[")),
        alt((
            map(pattern_rest, |r| Pattern::Array(vec![], r)),
            map(
                tuple((
                    separated_list0(ws(tag(",")), map(pattern, ArrayPatternItem::Pattern)),
                    opt(preceded(ws(tag(",")), opt(pattern_rest))),
                )),
                |(items, rest)| Pattern::Array(items, rest.flatten().unwrap_or(Rest::Exact)),
            ),
        )),
        ws(tag("]")),
    )(input)
}

fn pattern_capture<'v>(input: &str) -> IResult<&str, Pattern<'v>> {
    map(
        separated_pair(
            ws(identifier),
            ws(tag("@")),
            alt((pattern_atom, pattern_array, pattern_object)),
        ),
        |(id, pat)| Pattern::Capture(id, Box::new(pat)),
    )(input)
}

fn pattern_atom<'v>(input: &str) -> IResult<&str, Pattern<'v>> {
    map(literal, Pattern::Literal)(input)
}

pub fn pattern<'v>(input: &str) -> IResult<&str, Pattern<'v>> {
    alt((
        pattern_atom,
        pattern_capture,
        pattern_array,
        pattern_typed_identifier,
        pattern_typed_discard,
        pattern_identifier,
        pattern_discard,
        pattern_object,
    ))(input)
}

pub fn full_pattern<'v>(input: &str) -> Option<Pattern<'v>> {
    match all_consuming(pattern)(input) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}
