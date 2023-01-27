use damasc_lang::{
    parser::{
        assignment::assignment_set1,
        expression::expression_many1,
        io::{ParserError, ParserInput, ParserResult},
        util::ws,
    },
    syntax::assignment::AssignmentSet,
};
use damasc_query::parser::transformation;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    combinator::{all_consuming, cut, map, value},
    error::{context, convert_error, VerboseError},
    sequence::{pair, preceded},
    Finish,
};

use crate::command::Command;

pub(crate) fn command<'a, 'b, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Command<'a, 'b>, E> {
    context(
        "command",
        alt((
            value(Command::Cancel, all_consuming(space0)),
            value(
                Command::Help,
                all_consuming(context("cmd_help", alt((tag(".help"), tag(".h"))))),
            ),
            value(
                Command::Exit,
                all_consuming(context(
                    "cmd_exit",
                    alt((tag(".exit"), tag(".quit"), tag(".q"))),
                )),
            ),
            value(
                Command::ShowEnv,
                all_consuming(context("cmd_env", alt((tag(".env"), tag(".e"))))),
            ),
            value(
                Command::ClearEnv,
                all_consuming(context("cmd_clearenv", alt((tag(".clearenv"), tag(".ce"))))),
            ),
            cut(alt((
                context(
                    "cmd_transform",
                    map(all_consuming(transformation), Command::Transform),
                ),
                context(
                    "cmd_let",
                    map(
                        all_consuming(preceded(ws(tag("let ")), assignment_set1)),
                        Command::Assign,
                    ),
                ),
                context(
                    "binding",
                    map(all_consuming(assignment_set1), Command::Match),
                ),
                context(
                    "binding_expressions",
                    map(
                        all_consuming(pair(
                            expression_many1,
                            preceded(ws(tag("with ")), assignment_set1),
                        )),
                        |(expression, assignments)| Command::Eval(assignments, expression),
                    ),
                ),
                context(
                    "expressions",
                    all_consuming(map(expression_many1, |expression| {
                        Command::Eval(AssignmentSet::default(), expression)
                    })),
                ),
            ))),
        )),
    )(input)
}

pub fn command_all_consuming<'a, 'b>(input: &str) -> Result<Command<'a, 'b>, String> {
    match all_consuming(command::<VerboseError<ParserInput>>)(ParserInput::new(input)).finish() {
        Ok((_, s)) => Ok(s),
        Err(e) => {
            let errors = e
                .errors
                .into_iter()
                .filter_map(|(input, error)| match error {
                    nom::error::VerboseErrorKind::Context(_) => Some((*input.fragment(), error)),
                    nom::error::VerboseErrorKind::Char(_) => Some((*input.fragment(), error)),
                    nom::error::VerboseErrorKind::Nom(_) => None,
                })
                .collect();

            Err(convert_error(input, VerboseError { errors }))
        }
    }
}
