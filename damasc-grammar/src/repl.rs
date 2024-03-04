
use crate::query::single_transformation;
use crate::assignment::assignment_set_non_empty;
use damasc_lang::syntax::assignment::AssignmentSet;
use crate::expression::expression_set_non_empty;
use chumsky::prelude::choice;
use chumsky::prelude::just;
use damasc_repl::command::Command;
use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

pub fn single_command<'s,'a,'b>() -> impl Parser<'s, &'s str, Command<'a,'b>, extra::Err<Rich<'s, char>>> {
    choice((
    	just(".help").map(|_| Command::Help),
    	just(".h").map(|_| Command::Help),
    	just(".quit").map(|_| Command::Exit),
    	just(".q").map(|_| Command::Exit),
    	just(".exit").map(|_| Command::Exit),
    	just(".env").map(|_| Command::ShowEnv),
    	just(".clearenv").map(|_| Command::ClearEnv),
    	just(".ce").map(|_| Command::ClearEnv),
    	just(".pipe").padded().ignore_then(single_transformation()).map(Command::Transform),

    	just("let").padded().ignore_then(assignment_set_non_empty().then(just("with").padded().ignore_then(assignment_set_non_empty()).or_not()).map(|(assignments, locals)| {
    		Command::Assign(assignments, locals)
    	})),
    	assignment_set_non_empty().map(|assgns| Command::Match(assgns)),

    	expression_set_non_empty().then(just("with").ignore_then(assignment_set_non_empty().padded()).or_not()).map(|(exprs, assgns)| {
    		Command::Eval(assgns.unwrap_or_else(|| AssignmentSet::default()), exprs)
    	}),

    )).padded()
}