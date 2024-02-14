use std::collections::HashMap;

use damasc_lang::{
    identifier::Identifier,
    literal::Literal,
    parser::{
        assignment::assignment_set1,
        expression::{expression, expression_many1},
        identifier::{self, identifier},
        io::{ParserError, ParserInput, ParserResult},
        util::ws,
        value::value_bag,
    },
    syntax::expression::Expression,
};
use damasc_query::parser::multi_predicate;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map, opt, value},
    error::Error,
    multi::{fold_many0, many0},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};

use crate::{
    bag::Bag,
    bag_bundle::BagBundle,
    join::{Join, JoinSink, JoinSource},
};

pub fn bag_bundle<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<BagBundle<'_, '_>, E> {
    let (leftover, bags) = fold_many0(
        pair(
            ws(delimited(tag("$"), identifier::identifier, tag(":"))),
            value_bag,
        ),
        || Ok(HashMap::<Identifier, Bag>::new()),
        |acc, (id, values)| {
            acc.and_then(|mut h| {
                h.try_insert(id, values.into()).map_err(|_e| {
                    nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::Count))
                })?;

                Ok(h)
            })
        },
    )(input)?;

    bags.map(|bags| (leftover, BagBundle { bags }))
}

pub fn bag_bundle_all_consuming(bundle_string: &str) -> Option<BagBundle<'_, '_>> {
    match all_consuming(ws(bag_bundle::<Error<ParserInput>>))(ParserInput::new(bundle_string)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

fn join_source<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<JoinSource<'_, '_>, E> {
    alt((
        map(preceded(tag("$"), identifier), JoinSource::Named),
        map(preceded(tag("$!"), value_bag), JoinSource::Constant),
    ))(input)
}

fn join_sink<'v, 's, E: ParserError<'s>>(input: ParserInput<'s>) -> ParserResult<JoinSink<'_>, E> {
    alt((
        map(preceded(tag("$"), identifier), JoinSink::Named),
        value(JoinSink::Print, preceded(tag("$!"), ws(tag("print")))),
    ))(input)
}

fn join<'v, 's, E: ParserError<'s>>(input: ParserInput<'s>) -> ParserResult<Join<'_, '_>, E> {
    let (rest, (ins, outs, assigns, guard)) = ws(tuple((
        many0(separated_pair(join_source, ws(tag(">>")), multi_predicate)),
        many0(separated_pair(join_sink, ws(tag("<<")), expression_many1)),
        opt(preceded(ws(tag("with ")), assignment_set1)),
        opt(preceded(ws(tag("guard ")), expression)),
    )))(input)?;

    Ok((
        rest,
        Join {
            input: ins
                .into_iter()
                .try_fold(HashMap::new(), |mut acc, (id, pred)| {
                    if acc.try_insert(id, pred).is_ok() {
                        Ok(acc)
                    } else {
                        Err(nom::Err::Error(E::from_char(input, 'x')))
                    }
                })?,
            output: outs
                .into_iter()
                .try_fold(HashMap::new(), |mut a, (id, expr)| {
                    if a.try_insert(id, expr).is_ok() {
                        Ok(a)
                    } else {
                        Err(nom::Err::Error(E::from_char(input, 'x')))
                    }
                })?,
            local_assignments: assigns.unwrap_or_default(),
            guard: guard.unwrap_or(Expression::Literal(Literal::Boolean(true))),
        },
    ))
}

pub fn join_all_consuming(bundle_string: &str) -> Option<Join<'_, '_>> {
    match all_consuming(join::<Error<ParserInput>>)(ParserInput::new(bundle_string)) {
        Ok((_, r)) => Some(r),
        Err(e) => {
            dbg!(e);
            None
        }
    }
}
