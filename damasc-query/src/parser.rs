use damasc_lang::{parser::{util::ws, expression::{multi_expressions, expression}, pattern::{multi_patterns}}, syntax::{expression::{ExpressionSet, Expression}, pattern::{Pattern, PatternSet}}, literal::Literal, identifier::Identifier};
use nom::{sequence::{delimited, separated_pair, preceded, tuple, pair}, IResult, bytes::complete::tag, combinator::{map, opt, all_consuming}};

use crate::{transformation::Transformation, projection::MultiProjection, predicate::{MultiPredicate}, capture::MultiCapture};


pub fn bag(input: &str) -> IResult<&str, ExpressionSet> {
    delimited(
        ws(tag(r"{")), 
        multi_expressions, 
        ws(tag(r"}"))
    )(input)
}

pub fn projection(input: &str) -> IResult<&str, MultiProjection> {
    map(preceded(ws(tag("|>")), tuple((
        opt(preceded(ws(tag("map")), ws(multi_patterns))),
        opt(preceded(ws(tag("where")), ws(expression))),
        opt(preceded(ws(tag("into")), ws(multi_expressions)))
    ))), |(patterns, guard, proj)| {
        let pats = patterns.unwrap_or(PatternSet {
            patterns: vec![Pattern::Discard]
        });
        let auto_named_pats = PatternSet {
            patterns: pats.patterns.into_iter().enumerate().map(|(i, p)| Pattern::Capture(Identifier::new_owned(format!("${i}")), Box::new(p))).collect(),
        };
        let auto_projection = ExpressionSet { 
            expressions: auto_named_pats.patterns.iter().enumerate().map(|(i, p)| Expression::Identifier(Identifier::new_owned(format!("${i}")))).collect()
        };
        MultiProjection {
            predicate: MultiPredicate {
                capture: MultiCapture{ patterns: auto_named_pats },
                guard: guard.unwrap_or(Expression::Literal(Literal::Boolean(true))),
            },
            projections: proj.unwrap_or(auto_projection),
        }
    })(input)
}

fn transformation(input: &str) -> IResult<&str, Transformation> {
    all_consuming(map(pair(
        bag, opt(projection))
    , |(bag, projection)| Transformation {
        bag, projection: projection.unwrap_or_default()
    }))(input)
}

pub fn single_transformation(input: &str) -> Option<Transformation> {
    match all_consuming(transformation)(input) {
        Ok((_,r)) => Some(r),
        Err(e) => {
            dbg!(e);
            None
        },
    }
}