use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::map,
    multi::separated_list1,
    sequence::{separated_pair, terminated},
    IResult,
};

use crate::syntax::assignment::{Assignment, AssignmentSet};

use super::{expression::single_expression, pattern::pattern, util::ws};

pub fn assignment_set<'v, 'w>(input: &str) -> IResult<&str, AssignmentSet<'v, 'w>> {
    map(
        terminated(
            separated_list1(
                ws(tag(";")),
                map(
                    separated_pair(pattern, ws(tag("=")), single_expression),
                    |(pattern, expression)| Assignment {
                        pattern,
                        expression,
                    },
                ),
            ),
            alt((ws(tag(";")), space0)),
        ),
        |assignments| AssignmentSet { assignments },
    )(input)
}

pub fn assignment<'v, 'w>(input: &str) -> IResult<&str, Assignment<'v, 'w>> {
    map(
        separated_pair(pattern, ws(tag("=")), single_expression),
        |(pattern, expression)| Assignment {
            pattern,
            expression,
        },
    )(input)
}
