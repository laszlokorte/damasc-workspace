use damasc_lang::{parser::{util::ws, expression::{multi_expressions, expression, multi0_expressions}, pattern::{multi_patterns}}, syntax::{expression::{ExpressionSet, Expression}, pattern::{Pattern, PatternSet}}, literal::Literal, identifier::Identifier};
use nom::{sequence::{delimited, preceded, tuple, pair}, IResult, bytes::complete::tag, combinator::{map, opt, all_consuming}};

use crate::{transformation::Transformation, projection::MultiProjection, predicate::{MultiPredicate}, capture::MultiCapture};


pub fn bag<'s>(input: &str) -> IResult<&str, ExpressionSet<'s>> {
    delimited(
        ws(tag(r"{")), 
        multi_expressions, 
        ws(tag(r"}"))
    )(input)
}

pub fn bag_allow_empty(input: &str) -> IResult<&str, ExpressionSet> {
    delimited(
        ws(tag(r"{")), 
        multi0_expressions, 
        ws(tag(r"}"))
    )(input)
}

pub fn projection<'s>(input: &str) -> IResult<&str, MultiProjection<'s>> {
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
            expressions: (0..auto_named_pats.patterns.len()).map(|i| Expression::Identifier(Identifier::new_owned(format!("${i}")))).collect()
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

pub fn transformation<'a,'b>(input: &str) -> IResult<&str, Transformation<'a,'b>> {
    all_consuming(map(pair(
        bag, opt(projection))
    , |(bag, projection)| Transformation {
        bag, projection: projection.unwrap_or_default()
    }))(input)
}

pub fn single_transformation<'a,'b>(input: &str) -> Option<Transformation<'a,'b>> {
    match all_consuming(transformation)(input) {
        Ok((_,r)) => Some(r),
        Err(e) => {
            dbg!(e);
            None
        },
    }
}


pub fn single_bag(input: &str) -> Option<ExpressionSet> {
    match all_consuming(bag)(input) {
        Ok((_,r)) => Some(r),
        Err(e) => {
            dbg!(e);
            None
        },
    }
}

pub fn single_bag_allow_empty(input: &str) -> Option<ExpressionSet> {
    match all_consuming(bag_allow_empty)(input) {
        Ok((_,r)) => Some(r),
        Err(e) => {
            dbg!(e);
            None
        },
    }
}
