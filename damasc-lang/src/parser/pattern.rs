use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{all_consuming, map, opt, value},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair, tuple}, error::context,
};

use crate::syntax::{
    expression::PropertyKey,
    pattern::{
        ArrayPatternItem, ObjectPropertyPattern, Pattern, PatternSet, PropertyPattern, Rest,
    },
};

use super::{
    expression::expression,
    identifier::identifier,
    literal::{literal, literal_type_raw},
    util::ws, io::{ParserResult, ParserInput},
};

fn pattern_discard<'v>(input: ParserInput) -> ParserResult<Pattern<'v>> {
    context("pattern_discard", value(Pattern::Discard, tag("_")))(input)
}

fn pattern_typed_discard<'v>(input: ParserInput) -> ParserResult<Pattern<'v>> {
    context("pattern_typed_discard", map(
        preceded(ws(tag("_ is ")), literal_type_raw),
        Pattern::TypedDiscard,
    ))(input)
}

fn pattern_identifier<'v>(input: ParserInput) -> ParserResult<Pattern<'v>> {
    context("pattern_identifier", map(identifier, Pattern::Identifier))(input)
}

fn pattern_typed_identifier<'v>(input: ParserInput) -> ParserResult<Pattern<'v>> {
    context("pattern_typed_identifier", map(
        separated_pair(identifier, tag(" is "), literal_type_raw),
        |(i, t)| Pattern::TypedIdentifier(i, t),
    ))(input)
}

fn pattern_object_prop<'v>(input: ParserInput) -> ParserResult<ObjectPropertyPattern<'v>> {
    context("pattern_object_prop", alt((
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
    )))(input)
}

fn pattern_object<'v>(input: ParserInput) -> ParserResult<Pattern<'v>> {
    context("pattern_object", delimited(
        ws(tag("{")),
        alt((
            map(pattern_rest, |r| Pattern::Object(vec![], r)),
            map(
                tuple((
                    separated_list0(ws(ws(tag(","))), pattern_object_prop),
                    opt(preceded(ws(tag(",")), opt(pattern_rest))),
                )),
                |(props, rest)| Pattern::Object(props, rest.flatten().unwrap_or(Rest::Exact)),
            ),
        )),
        ws(tag("}")),
    ))(input)
}

fn pattern_rest<'v>(input: ParserInput) -> ParserResult<Rest<'v>> {
    context("pattern_rest", alt((
        map(preceded(ws(tag("...")), pattern), |r| {
            Rest::Collect(Box::new(r))
        }),
        value(Rest::Discard, ws(tag("..."))),
    )))(input)
}

fn pattern_array<'v>(input: ParserInput) -> ParserResult<Pattern<'v>> {
    context("pattern_array", delimited(
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
    ))(input)
}

fn pattern_capture<'v>(input: ParserInput) -> ParserResult<Pattern<'v>> {
    context("pattern_capture", map(
        separated_pair(
            ws(identifier),
            ws(tag("@")),
            alt((pattern_atom, pattern_array, pattern_object)),
        ),
        |(id, pat)| Pattern::Capture(id, Box::new(pat)),
    ))(input)
}

fn pattern_atom<'v>(input: ParserInput) -> ParserResult<Pattern<'v>> {
    context("pattern_atom", map(literal, Pattern::Literal))(input)
}

pub fn pattern<'v>(input: ParserInput) -> ParserResult<Pattern<'v>> {
    context("pattern", alt((
        pattern_atom,
        pattern_capture,
        pattern_array,
        pattern_typed_identifier,
        pattern_typed_discard,
        pattern_identifier,
        pattern_discard,
        pattern_object,
    )))(input)
}

pub fn pattern_all_consuming<'v>(input: &str) -> Option<Pattern<'v>> {
    match all_consuming(pattern)(ParserInput::new(input)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

pub fn many0_pattern<'v>(input: ParserInput) -> ParserResult<PatternSet<'v>> {
    delimited(
        space0,
        map(separated_list0(ws(tag(";")), pattern), |patterns| {
            PatternSet { patterns }
        }),
        ws(opt(tag(";"))),
    )(input)
}


pub fn many1_pattern<'v>(input: ParserInput) -> ParserResult<PatternSet<'v>> {
    delimited(
        space0,
        map(separated_list1(ws(tag(";")), pattern), |patterns| {
            PatternSet { patterns }
        }),
        ws(opt(tag(";"))),
    )(input)
}
