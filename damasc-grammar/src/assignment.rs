use crate::expression::decl_single_expression;
use crate::pattern::decl_single_pattern;
use chumsky::extra;
use chumsky::prelude::any;
use chumsky::prelude::choice;
use chumsky::prelude::just;
use chumsky::prelude::skip_then_retry_until;
use chumsky::prelude::Rich;
use chumsky::IterParser;
use damasc_lang::syntax::assignment::Assignment;
use damasc_lang::syntax::assignment::AssignmentSet;

use chumsky::Parser;

pub fn single_assignment<'s,'a,'b>(
) -> impl Parser<'a, &'a str, Assignment<'a, 'a>, extra::Err<Rich<'a, char>>> {
    let (single_pat, mut expr_decl) = decl_single_pattern();
    let (single_expr, mut pat_decl) = decl_single_expression();

    expr_decl.define(single_expr.clone());
    pat_decl.define(single_pat.clone());

    single_pat.padded()
        .labelled("pattern")
        .as_context()
        .then_ignore(just("=").padded())
        .then(single_expr.padded().labelled("expression").as_context())
        .map(|(pattern, expression)| Assignment {
            pattern,
            expression,
        })
        .labelled("assignment")
        .as_context()
}

pub fn assignment_set<'s,'a,'b>(
) -> impl Parser<'s, &'s str, AssignmentSet<'a, 'b>, extra::Err<Rich<'s, char>>> {
    single_assignment()
        .separated_by(just(';').padded().recover_with(skip_then_retry_until(
            any().ignored(),
            choice((just(";"),)).ignored(),
        )))
        .allow_trailing()
        .collect()
        .map(|assignments| AssignmentSet { assignments })
        .labelled("assignment_set")
        .as_context()
        .map(move |a|a.deep_clone())
}

pub fn assignment_set_non_empty<'s,'a,'b>(
) -> impl Parser<'s, &'s str, AssignmentSet<'a, 'b>, extra::Err<Rich<'s, char>>> {
    single_assignment()
        .separated_by(just(';').padded().recover_with(skip_then_retry_until(
            any().ignored(),
            choice((just(";"),)).ignored(),
        )))
        .allow_trailing()
        .at_least(1)
        .collect()
        .map(|assignments| AssignmentSet { assignments })
        .labelled("assignment_set")
        .as_context()
        .map(move |a|a.deep_clone())
}
