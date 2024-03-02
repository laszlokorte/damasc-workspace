use damasc_lang::syntax::expression::LambdaAbstraction;
use damasc_lang::syntax::expression::IfElseExpression;
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
        let boxed_pattern = single_pattern().map(Box::new);
        // let object = ...;
        // let array = ...;
        // let abstraction = ...;
        // let maching = ...;
        // let condition = ...;

        let literal = single_literal().map_with(|l, meta| Expression::new_with_location(ExpressionBody::Literal(l), meta_to_location(meta))).boxed();

        let matching_case = single_pattern()
            .labelled("case_pattern")
            .as_context()
            .then(just("if").padded().ignore_then(boxed_expression.clone()).or_not())
            .then_ignore(just("=>").padded())
            .then(boxed_expression.clone())
            .map(|((pattern, guard), body)| MatchCase { pattern, body, guard })
            .boxed();

        let matching = just("match").padded().ignore_then(boxed_expression.clone()).then(matching_case
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

        let condition = just("if").padded().ignore_then(boxed_expression.clone().labelled("if_condition")).then(boxed_expression.clone()
            .delimited_by(
                just('{'),
                just('}')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            )
            .labelled("if_body")
            .as_context()).then(just("else").padded().ignore_then(boxed_expression.clone().labelled("else_body").as_context())
            .delimited_by(
                just('{'),
                just('}')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            ).or_not())
            .map_with(|((condition, true_branch), false_branch), meta| 
                Expression::new_with_location(
                    ExpressionBody::Condition(
                        IfElseExpression{condition, true_branch, false_branch}), meta_to_location(meta)
                    )).boxed();

        let abstraction = just("fn")
        .ignore_then(single_pattern())
        .then(just("=>").padded().ignore_then(boxed_expression.clone()))
        .map_with(|(arguments, body), meta| Expression::new_with_location(ExpressionBody::Abstraction(LambdaAbstraction {
                    arguments,
                    body,
                }), meta_to_location(meta))).boxed();

        choice((abstraction, matching, condition, literal))
        .padded()
    })
}
