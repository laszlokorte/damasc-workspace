
use damasc_lang::{parser::expression::{expression, multi_expressions}, syntax::assignment::AssignmentSet};
use damasc_query::parser::transformation;
use nom::{IResult, branch::alt, bytes::complete::tag, combinator::{value, map, all_consuming}};

use crate::command::Command;

pub(crate) fn command(input:&str) -> IResult<&str, Command> {
    alt((
        value(Command::Help, alt((tag(".help"), tag(".h")))),
        value(Command::Exit, alt((tag(".exit"), tag(".quit")))),
        map(transformation, Command::Transform),
        map(multi_expressions, |e| Command::Eval(AssignmentSet::default(), e)),
    ))(input)
}

pub fn full_command(input: &str) -> Option<Command> {
    let Ok((_, r)) = all_consuming(command)(input) else {
        return None;
    };

    return Some(r)
}