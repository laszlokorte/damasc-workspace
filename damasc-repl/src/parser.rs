use damasc_lang::parser::{assignment::assignment_set1, expression::expression_many1, util::ws};
use damasc_query::parser::transformation;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map, opt, value},
    sequence::{pair, preceded},
    IResult, character::complete::space0, error::context,
};

use crate::command::Command;

pub(crate) fn command<'a, 'b>(input: &str) -> IResult<&str, Command<'a, 'b>> {
    context("command", alt((
        value(Command::Help, alt((tag(".help"), tag(".h")))),
        value(Command::Exit, alt((tag(".exit"), tag(".quit")))),
        value(Command::ShowEnv, tag(".env")),
        map(preceded(ws(tag("let ")), assignment_set1), Command::Assign),
        map(assignment_set1, Command::Match),
        map(
            pair(
                expression_many1,
                opt(preceded(ws(tag("with ")), assignment_set1)),
            ),
            |(expression, assignments)| Command::Eval(assignments.unwrap_or_default(), expression),
        ),
        map(transformation, Command::Transform),
        value(Command::Cancel, space0),
    )))(input)
}

pub fn command_all_consuming<'a, 'b>(input: &str) -> Option<Command<'a, 'b>> {
    let Ok((_, r)) = all_consuming(command)(input) else {
        return None;
    };

    Some(r)
}
