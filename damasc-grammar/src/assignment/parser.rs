use crate::expression::parser::single_expression;
use crate::pattern::parser::single_pattern;
use chumsky::extra;
use chumsky::prelude::just;
use chumsky::prelude::Rich;
use damasc_lang::syntax::assignment::Assignment;

use chumsky::Parser;

pub fn single_assignment<'a>(
) -> impl Parser<'a, &'a str, Assignment<'a, 'a>, extra::Err<Rich<'a, char>>> {
    single_pattern().labelled("pattern").as_context()
        .then_ignore(just("=").padded())
        .then(single_expression().labelled("expression").as_context())
        .map(|(pattern, expression)| Assignment {
            pattern,
            expression,
        }).labelled("assignment").as_context()
}
