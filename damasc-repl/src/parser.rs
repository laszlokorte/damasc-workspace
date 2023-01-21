use damasc_lang::parser::{assignment::assignment_set1, expression::expression_many1, util::ws, io::{ParserResult, ParserInput}};
use damasc_query::parser::transformation;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map, opt, value, cut},
    sequence::{pair, preceded}, character::complete::space0, error::context, Finish,
};

use crate::command::Command;

pub(crate) fn command<'a, 'b>(input: ParserInput) -> ParserResult<Command<'a, 'b>> {
    context("command", alt((
        value(Command::Cancel, all_consuming(space0)),
        value(Command::Help, alt((tag(".help"), tag(".h")))),
        value(Command::Exit, alt((tag(".exit"), tag(".quit")))),
        value(Command::ShowEnv, tag(".env")),

        map(preceded(ws(tag("let ")), cut(assignment_set1)), Command::Assign),
        map(assignment_set1, Command::Match),
        map(
            cut(pair(
                expression_many1,
                opt(preceded(ws(tag("with ")), assignment_set1)),
            )),
            |(expression, assignments)| Command::Eval(assignments.unwrap_or_default(), expression),
        ),
        map(cut(transformation), Command::Transform),
    )))(input)
}

pub fn command_all_consuming<'a, 'b>(input: &str) -> Result<Command<'a, 'b>, String> {
    match all_consuming(command)(ParserInput::new(input)).finish() {
        Ok((_, s)) => Ok(s),
        Err(e) => Err(e.to_string()),
    }
}