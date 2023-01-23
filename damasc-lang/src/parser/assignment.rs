use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{map, all_consuming},
    multi::{separated_list1, separated_list0},
    sequence::{separated_pair, terminated}, error::{context, Error},
};

use crate::syntax::assignment::{Assignment, AssignmentSet};

use super::{expression::expression, pattern::pattern, util::ws, io::{ParserResult, ParserInput, ParserError}};

pub fn assignment_set0<'v, 'w, 's, E:ParserError<'s>>(input: ParserInput<'s>) -> ParserResult<AssignmentSet<'v, 'w>, E> {
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


pub fn assignment_set1<'v, 'w, 's, E:ParserError<'s>>(input: ParserInput<'s>) -> ParserResult<AssignmentSet<'v, 'w>, E> {
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

pub fn assignment<'v, 'w, 's, E:ParserError<'s>>(input: ParserInput<'s>) -> ParserResult<Assignment<'v, 'w>, E> {
    context("assignment", map(
        separated_pair(pattern, ws(tag("=")), expression),
        |(pattern, expression)| Assignment {
            pattern,
            expression,
        },
    ))(input)
}

pub fn assignment_all_consuming(input: ParserInput) -> Option<Assignment<'_, '_>> {
    let Ok((_,r)) = all_consuming(assignment::<Error<ParserInput>>)(input) else {
        return None
    };
    Some(r)
}
pub fn assignment_set1_all_consuming(input: &str) -> Option<AssignmentSet<'_, '_>> {
    let Ok((_,r)) = all_consuming(assignment_set1::<Error<ParserInput>>)(ParserInput::new(input)) else {
        return None
    };
    Some(r)
}