use damasc_lang::{parser::{assignment::assignment_set1, expression::expression_many1, util::ws, io::{ParserResult, ParserInput}}, syntax::assignment::AssignmentSet};
use damasc_query::parser::transformation;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map, value},
    sequence::{pair, preceded}, character::complete::space0, error::context, Finish,
};

use crate::command::Command;

pub(crate) fn command<'a, 'b>(input: ParserInput) -> ParserResult<Command<'a, 'b>> {
    context("command", alt((
        value(Command::Cancel, all_consuming(space0)),
        value(Command::Help, all_consuming(alt((tag(".help"), tag(".h"))))),
        value(Command::Exit, all_consuming(alt((tag(".exit"), tag(".quit"), tag(".q"))))),
        value(Command::ShowEnv, all_consuming(alt((tag(".env"), tag(".e"))))),
        value(Command::ClearEnv, all_consuming(alt((tag(".clearenv"), tag(".ce"))))),
        map(all_consuming(transformation), Command::Transform),
        map(all_consuming(preceded(ws(tag("let ")), assignment_set1)), Command::Assign),
        map(all_consuming(assignment_set1), Command::Match),
        map(all_consuming(
            pair(
                expression_many1,
                preceded(ws(tag("with ")), assignment_set1),
            )),
            |(expression, assignments)| Command::Eval(assignments, expression),
        ),
        map(
            all_consuming(expression_many1),
            |expression| Command::Eval(AssignmentSet::default(), expression),
        ),
    )))(input)
}

pub fn command_all_consuming<'a, 'b>(input: &str) -> Result<Command<'a, 'b>, String> {
    match all_consuming(command)(ParserInput::new(input)).finish() {
        Ok((_, s)) => Ok(s),
        Err(e) => {
            Err(e.to_string())
        },
    }
}