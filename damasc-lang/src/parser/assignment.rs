use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{all_consuming, map},
    error::{context, Error},
    multi::{separated_list0, separated_list1},
    sequence::{separated_pair, terminated},
    Parser,
};

use crate::syntax::assignment::{Assignment, AssignmentSet};

use super::{
    expression::expression,
    io::{ParserError, ParserInput, ParserResult},
    pattern::pattern,
    util::ws,
};

pub fn assignment_set0<'v, 'w, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<'s, AssignmentSet<'v, 'w>, E> {
    context(
        "assignment_set",
        map(
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
        ),
    )
    .parse(input)
}

pub fn assignment_set1<'v, 'w, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<'s, AssignmentSet<'v, 'w>, E> {
    context(
        "assignment_set",
        map(
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
        ),
    )
    .parse(input)
}

pub fn assignment<'v, 'w, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<'s, Assignment<'v, 'w>, E> {
    context(
        "assignment",
        map(
            separated_pair(pattern, ws(tag("=")), expression),
            |(pattern, expression)| Assignment {
                pattern,
                expression,
            },
        ),
    )
    .parse(input)
}

pub fn assignment_all_consuming(input: ParserInput<'_>) -> Option<Assignment<'_, '_>> {
    let Ok((_, r)) = all_consuming(assignment::<Error<ParserInput>>).parse(input) else {
        return None;
    };
    Some(r)
}
pub fn assignment_set1_all_consuming(input: &str) -> Option<AssignmentSet<'_, '_>> {
    let Ok((_, r)) =
        all_consuming(assignment_set1::<Error<ParserInput>>).parse(ParserInput::new(input))
    else {
        return None;
    };
    Some(r)
}
