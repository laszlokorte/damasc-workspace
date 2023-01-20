use damasc_lang::parser::{assignment::assignment_set, expression::multi_expressions, util::ws};
use damasc_query::parser::transformation;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map, opt, value},
    sequence::{pair, preceded},
    IResult,
};

use crate::command::Command;

pub(crate) fn command<'a, 'b>(input: &str) -> IResult<&str, Command<'a, 'b>> {
    alt((
        value(Command::Help, alt((tag(".help"), tag(".h")))),
        value(Command::Exit, alt((tag(".exit"), tag(".quit")))),
        map(preceded(ws(tag("let ")), assignment_set), Command::Assign),
        map(assignment_set, Command::Match),
        map(
            pair(
                multi_expressions,
                opt(preceded(ws(tag("with ")), assignment_set)),
            ),
            |(expression, assignments)| Command::Eval(assignments.unwrap_or_default(), expression),
        ),
        map(transformation, Command::Transform),
    ))(input)
}

pub fn full_command<'a, 'b>(input: &str) -> Option<Command<'a, 'b>> {
    let Ok((_, r)) = all_consuming(command)(input) else {
        return None;
    };

    Some(r)
}
