use nom::{IResult, sequence::{terminated, separated_pair}, combinator::map, multi::separated_list1, bytes::complete::tag, character::complete::space0, branch::alt};

use crate::syntax::assignment::{AssignmentSet, Assignment};

use super::{util::ws, pattern::pattern, expression::single_expression};

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
