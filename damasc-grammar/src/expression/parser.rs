use damasc_lang::syntax::expression::MatchCase;
use damasc_lang::syntax::expression::MatchExpression;
use crate::pattern::parser::single_pattern;
use crate::util::meta_to_location;
use damasc_lang::syntax::expression::ExpressionBody;
use damasc_lang::syntax::expression::Expression;

use crate::literal::parser::single_literal;

use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

use chumsky::prelude::*;

pub fn single_expression<'s>() -> impl Parser<'s, &'s str, Expression<'s>, extra::Err<Rich<'s, char>>>  {
    recursive(|expression| {
        let boxed_expression = expression.map(Box::new);
        // let object = ...;
        // let array = ...;
        // let abstraction = ...;
        // let maching = ...;
        // let condition = ...;

        let literal = single_literal().map_with(|l, meta| Expression::new_with_location(ExpressionBody::Literal(l), meta_to_location(meta))).boxed();

        let matching_case = single_pattern()
            .labelled("case_pattern")
            .as_context()
            .then_ignore(just("=>").padded())
            .then(boxed_expression.clone())
            .map(|(pattern, body)| MatchCase { pattern, body, guard: None })
            .boxed();

        let matching = just("match").padded().ignore_then(boxed_expression).then(matching_case
            .separated_by(just(',').padded().recover_with(skip_then_retry_until(
                any().ignored(),
                one_of(",}").ignored(),
            )))
            .allow_trailing()
            .collect()
            .padded()
            .delimited_by(
                just('{'),
                just('}')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            )
            .labelled("match")
            .as_context())
            .map_with(|(subject, cases), meta| 
                Expression::new_with_location(
                    ExpressionBody::Match(
                        MatchExpression{subject, cases}), meta_to_location(meta)
                    )).boxed();

        choice((matching, literal))
        .recover_with(skip_then_retry_until(
            any().ignored(),
            one_of("]}").ignored(),
        ))
        .padded()
    })
}
