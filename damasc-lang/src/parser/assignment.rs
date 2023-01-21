use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{map, all_consuming},
    multi::{separated_list1, separated_list0},
    sequence::{separated_pair, terminated},
    IResult, error::context,
};

use crate::syntax::assignment::{Assignment, AssignmentSet};

use super::{expression::expression, pattern::pattern, util::ws};

pub fn assignment_set0<'v, 'w>(input: &str) -> IResult<&str, AssignmentSet<'v, 'w>> {
    context("assignment_set", map(
        terminated(
            separated_list0(
                ws(tag(";")),
                map(
                    separated_pair(pattern, ws(tag("=")), expression),
                    |(pattern, expression)| Assignment {
                        pattern,
                        expression,
                    },
                ),
            ),
            alt((ws(tag(";")), space0)),
        ),
        |assignments| AssignmentSet { assignments },
    ))(input)
}


pub fn assignment_set1<'v, 'w>(input: &str) -> IResult<&str, AssignmentSet<'v, 'w>> {
    context("assignment_set", map(
        terminated(
            separated_list1(
                ws(tag(";")),
                map(
                    separated_pair(pattern, ws(tag("=")), expression),
                    |(pattern, expression)| Assignment {
                        pattern,
                        expression,
                    },
                ),
            ),
            alt((ws(tag(";")), space0)),
        ),
        |assignments| AssignmentSet { assignments },
    ))(input)
}

pub fn assignment<'v, 'w>(input: &str) -> IResult<&str, Assignment<'v, 'w>> {
    context("assignment", map(
        separated_pair(pattern, ws(tag("=")), expression),
        |(pattern, expression)| Assignment {
            pattern,
            expression,
        },
    ))(input)
}

pub fn assignment_all_consuming(input: &str) -> Option<Assignment<'_, '_>> {
    let Ok((_,r)) = all_consuming(assignment)(input) else {
        return None
    };
    Some(r)
}
pub fn assignment_set1_all_consuming(input: &str) -> Option<AssignmentSet<'_, '_>> {
    let Ok((_,r)) = all_consuming(assignment_set1)(input) else {
        return None
    };
    Some(r)
}