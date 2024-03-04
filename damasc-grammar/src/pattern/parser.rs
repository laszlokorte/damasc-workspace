use chumsky::recursive::Indirect;
use damasc_lang::syntax::expression::ExpressionBody;
use damasc_lang::syntax::expression::Expression;
use crate::literal::parser::single_type_literal;
use crate::identifier::parser::single_identifier;
use crate::util::meta_to_location;
use damasc_lang::syntax::pattern::PatternBody;
use damasc_lang::syntax::pattern::Pattern;

use crate::literal::parser::single_literal;

use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

use chumsky::prelude::*;

pub fn single_pattern<'s>() -> (Boxed<'s, 's, &'s str, Pattern<'s>, extra::Err<Rich<'s, char>>>, Recursive<Indirect<'s, 's, &'s str, Expression<'s>, extra::Err<Rich<'s, char>>>>) {
    let expression_declaration = Recursive::declare();

    let pattern_declartion = recursive(|pattern| {
        let literal = single_literal().boxed().map_with(|l, meta| Pattern::new_with_location(PatternBody::Literal(l), meta_to_location(meta)))
        .labelled("literal")
        .as_context()
        .boxed();


        let discard = just("_").ignore_then(just("as").padded().ignore_then(single_type_literal().or_not())).map_with(|value_type, meta| {
            if let Some(t) = value_type {
                Pattern::new_with_location(PatternBody::TypedDiscard(t), meta_to_location(meta))
            } else {
                Pattern::new_with_location(PatternBody::Discard, meta_to_location(meta))
            }
        }).boxed();

        let capture = single_identifier().then(just("@").ignore_then(pattern).or_not()).boxed().map_with(|(id, pat), meta| {
            if let Some(p) = pat {
                Pattern::new_with_location(PatternBody::Capture(id, Box::new(p)), meta_to_location(meta))
            } else {
                Pattern::new_with_location(PatternBody::Identifier(id), meta_to_location(meta))
            }
        })
        .labelled("identifier")
        .as_context()
        .boxed();

        let typed_identifier = single_identifier().then(just("as").padded().ignore_then(single_type_literal())).map_with(|(id, value_type), meta| {
            Pattern::new_with_location(PatternBody::TypedIdentifier(id, value_type), meta_to_location(meta))
        }).boxed();

        let pinned = just("^").ignore_then(choice((
            single_identifier().map_with(|id, meta| {
                Expression::new_with_location(ExpressionBody::Identifier(id), meta_to_location(meta))
            }),
            expression_declaration.clone().delimited_by(
                just("("),
                just(")")
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),),
        ))).map_with(|expr, meta| {
            Pattern::new_with_location(PatternBody::PinnedExpression(Box::new(expr)), meta_to_location(meta))
        }).boxed();


        // PinnedExpression(Box<Expression<'s>>),
        // Literal(Literal<'s>),
        // Object(ObjectPattern<'s>, Rest<'s>),
        // Array(ArrayPattern<'s>, Rest<'s>),


        choice((
            literal,
            capture,
            discard,
            typed_identifier,
            pinned,
        ))
    }).padded().boxed();

    (pattern_declartion, expression_declaration)
}
