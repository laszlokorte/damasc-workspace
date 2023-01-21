use damasc_lang::{
    identifier::Identifier,
    literal::Literal,
    parser::{
        expression::{expression, expression_many0, expression_many1},
        pattern::many1_pattern,
        util::ws,
    },
    syntax::{
        expression::{Expression, ExpressionSet},
        pattern::{Pattern, PatternSet},
    },
};
use nom::{
    bytes::complete::tag,
    combinator::{all_consuming, map, opt},
    sequence::{delimited, pair, preceded, tuple},
    IResult, error::context,
};

use crate::{
    capture::MultiCapture, predicate::MultiPredicate, projection::MultiProjection,
    transformation::Transformation,
};

pub fn query_bag<'s>(input: &str) -> IResult<&str, ExpressionSet<'s>> {
    context("query_bag", delimited(ws(tag(r"{")), expression_many1, ws(tag(r"}"))))(input)
}

pub fn query_bag_allow_empty(input: &str) -> IResult<&str, ExpressionSet> {
    context("query_bag_allow_empty", delimited(ws(tag(r"{")), expression_many0, ws(tag(r"}"))))(input)
}

pub fn projection<'s>(input: &str) -> IResult<&str, MultiProjection<'s>> {
    context("query_projection", map(
        preceded(
            ws(tag("|>")),
            tuple((
                opt(preceded(ws(tag("map")), context("query_projection_map", ws(many1_pattern)))),
                opt(preceded(ws(tag("where")), context("query_projection_predicate", ws(expression)))),
                opt(preceded(ws(tag("into")), context("query_projection_expression", ws(expression_many1)))),
            )),
        ),
        |(patterns, guard, proj)| {
            let pats = patterns.unwrap_or(PatternSet {
                patterns: vec![Pattern::Discard],
            });
            let auto_named_pats = PatternSet {
                patterns: pats
                    .patterns
                    .into_iter()
                    .enumerate()
                    .map(|(i, p)| {
                        Pattern::Capture(Identifier::new_owned(format!("${i}")), Box::new(p))
                    })
                    .collect(),
            };
            let auto_projection = ExpressionSet {
                expressions: (0..auto_named_pats.patterns.len())
                    .map(|i| Expression::Identifier(Identifier::new_owned(format!("${i}"))))
                    .collect(),
            };
            MultiProjection {
                predicate: MultiPredicate {
                    capture: MultiCapture {
                        patterns: auto_named_pats,
                    },
                    guard: guard.unwrap_or(Expression::Literal(Literal::Boolean(true))),
                },
                projections: proj.unwrap_or(auto_projection),
            }
        },
    ))(input)
}

pub fn transformation<'a, 'b>(input: &str) -> IResult<&str, Transformation<'a, 'b>> {
    context("transformation", map(pair(query_bag, opt(projection)), |(bag, projection)| {
        Transformation {
            bag,
            projection: projection.unwrap_or_default(),
        }
    }))(input)
}

pub fn transformation_all_consuming<'a, 'b>(input: &str) -> Option<Transformation<'a, 'b>> {
    match all_consuming(transformation)(input) {
        Ok((_, r)) => Some(r),
        Err(e) => {
            dbg!(e);
            None
        }
    }
}

pub fn query_bag_all_consuming(input: &str) -> Option<ExpressionSet> {
    match all_consuming(query_bag)(input) {
        Ok((_, r)) => Some(r),
        Err(e) => {
            dbg!(e);
            None
        }
    }
}

pub fn query_bag_allow_empty_all_consuming(input: &str) -> Option<ExpressionSet> {
    match all_consuming(query_bag_allow_empty)(input) {
        Ok((_, r)) => Some(r),
        Err(e) => {
            dbg!(e);
            None
        }
    }
}
