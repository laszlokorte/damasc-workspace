use crate::parser::located::located_pattern;
use crate::parser::expression::expression_identifier;
use crate::syntax::pattern::PatternBody;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{all_consuming, map, opt, value},
    error::{context, Error},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair, tuple},
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
    io::{ParserError, ParserInput, ParserResult},
    literal::{literal, literal_type_raw},
    util::ws,
};

fn pattern_discard<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Pattern<'v>, E> {
    context(
        "pattern_discard",
        located_pattern(value(PatternBody::Discard, tag("_"))),
    )(input)
}

fn pattern_typed_discard<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Pattern<'v>, E> {
    context(
        "pattern_typed_discard",
        located_pattern(map(
            preceded(ws(tag("_ is ")), literal_type_raw),
            PatternBody::TypedDiscard,
        )),
    )(input)
}

fn pattern_identifier<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Pattern<'v>, E> {
    context(
        "pattern_identifier",
        located_pattern(map(identifier, PatternBody::Identifier)),
    )(input)
}

fn pattern_typed_identifier<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Pattern<'v>, E> {
    context(
        "pattern_typed_identifier",
        located_pattern(map(
            separated_pair(identifier, tag(" is "), literal_type_raw),
            |(i, t)| PatternBody::TypedIdentifier(i, t),
        )),
    )(input)
}

fn pattern_object_prop<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<ObjectPropertyPattern<'v>, E> {
    context(
        "pattern_object_prop",
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
        )),
    )(input)
}

fn pattern_object<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Pattern<'v>, E> {
    context(
        "pattern_object",
        located_pattern(
            delimited(
                ws(tag("{")),
                alt((
                    map(pattern_rest, |r| PatternBody::Object(vec![], r)),
                    map(
                        tuple((
                            separated_list0(ws(ws(tag(","))), pattern_object_prop),
                            opt(preceded(ws(tag(",")), opt(pattern_rest))),
                        )),
                        |(props, rest)| {
                            PatternBody::Object(props, rest.flatten().unwrap_or(Rest::Exact))
                        },
                    ),
                )),
                ws(tag("}")),
            )
        ),
    )(input)
}

fn pattern_rest<'v, 's, E: ParserError<'s>>(input: ParserInput<'s>) -> ParserResult<Rest<'v>, E> {
    context(
        "pattern_rest",
        alt((
            map(preceded(ws(tag("...")), pattern), |r| {
                Rest::Collect(Box::new(r))
            }),
            value(Rest::Discard, ws(tag("..."))),
        )),
    )(input)
}

fn pattern_array<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Pattern<'v>, E> {
    context(
        "pattern_array",
        located_pattern(
            delimited(
                ws(tag("[")),
                alt((
                    map(pattern_rest, |r| PatternBody::Array(vec![], r)),
                    map(
                        tuple((
                            separated_list0(ws(tag(",")), map(pattern, ArrayPatternItem::Pattern)),
                            opt(preceded(ws(tag(",")), opt(pattern_rest))),
                        )),
                        |(items, rest)| {
                            PatternBody::Array(items, rest.flatten().unwrap_or(Rest::Exact))
                        },
                    ),
                )),
                ws(tag("]")),
            )
        ),
    )(input)
}

fn pattern_capture<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Pattern<'v>, E> {
    context(
        "pattern_capture",
        located_pattern(map(
            separated_pair(
                ws(identifier),
                ws(tag("@")),
                alt((
                    pattern_atom,
                    pattern_array,
                    pattern_object,
                    pattern_pinned_expression,
                )),
            ),
            |(id, pat)| PatternBody::Capture(id, Box::new(pat)),
        )),
    )(input)
}

fn pattern_atom<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Pattern<'v>, E> {
    context(
        "pattern_atom",
        located_pattern(map(literal, PatternBody::Literal)),
    )(input)
}

fn pattern_pinned_expression<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Pattern<'v>, E> {
    context(
        "pattern_pinned_expression",
        located_pattern(map(
            preceded(
                ws(tag("^")),
                alt((
                    delimited(ws(tag("(")), expression, ws(tag(")"))),
                    expression_identifier,
                )),
            ),
            |expr| PatternBody::PinnedExpression(Box::new(expr)),
        )),
    )(input)
}

pub fn pattern<'v, 's, E: ParserError<'s>>(input: ParserInput<'s>) -> ParserResult<Pattern<'v>, E> {
    context(
        "pattern",
        alt((
            pattern_pinned_expression,
            pattern_atom,
            pattern_capture,
            pattern_array,
            pattern_typed_identifier,
            pattern_typed_discard,
            pattern_identifier,
            pattern_discard,
            pattern_object,
        )),
    )(input)
}

pub fn pattern_all_consuming<'v>(input: &str) -> Option<Pattern<'v>> {
    match all_consuming(pattern::<Error<ParserInput>>)(ParserInput::new(input)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

pub fn many0_pattern<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<PatternSet<'v>, E> {
    delimited(
        space0,
        map(separated_list0(ws(tag(";")), pattern), |patterns| {
            PatternSet { patterns }
        }),
        ws(opt(tag(";"))),
    )(input)
}

pub fn many1_pattern<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<PatternSet<'v>, E> {
    delimited(
        space0,
        map(separated_list1(ws(tag(";")), pattern), |patterns| {
            PatternSet { patterns }
        }),
        ws(opt(tag(";"))),
    )(input)
}
