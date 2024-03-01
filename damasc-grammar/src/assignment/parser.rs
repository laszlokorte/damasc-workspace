use crate::expression::parser::single_expression;
use chumsky::prelude::just;
use crate::pattern::parser::single_pattern;
use damasc_lang::syntax::assignment::Assignment;
use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

pub fn single_assignment<'a>(
) -> impl Parser<'a, &'a str, Assignment<'a,'a>, extra::Err<Rich<'a, char>>> {
    single_pattern()
    .then_ignore(just("=").padded())
    .then(single_expression())
    .map(|(pattern, expression)| 
        Assignment{ pattern, expression }
    )
}
