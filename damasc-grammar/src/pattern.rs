use damasc_lang::syntax::pattern::PatternSet;
use crate::expression::decl_single_expression;
use crate::identifier::single_identifier;
use crate::literal::single_string_literal;
use crate::literal::single_type_literal;
use crate::util::meta_to_location;
use chumsky::recursive::Indirect;
use damasc_lang::identifier::Identifier;
use damasc_lang::syntax::expression::Expression;
use damasc_lang::syntax::expression::ExpressionBody;
use damasc_lang::syntax::expression::PropertyKey;
use damasc_lang::syntax::pattern::ArrayPatternItem;
use damasc_lang::syntax::pattern::ObjectPropertyPattern;
use damasc_lang::syntax::pattern::Pattern;
use damasc_lang::syntax::pattern::PatternBody;
use damasc_lang::syntax::pattern::PropertyPattern;
use damasc_lang::syntax::pattern::Rest;

use crate::literal::single_literal;

use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

use chumsky::prelude::*;

type ExpressionParserDelc<'s,'a> = (
    Boxed<'s, 's, &'s str, Pattern<'a>, extra::Err<Rich<'s, char>>>,
    Recursive<Indirect<'s, 's, &'s str, Expression<'a>, extra::Err<Rich<'s, char>>>>,
);



pub fn pattern_set<'s>(
) -> impl Parser<'s, &'s str, PatternSet<'s>, extra::Err<Rich<'s, char>>> {
	single_pattern().separated_by(just(';').padded().recover_with(skip_then_retry_until(
        any().ignored(),
        one_of(";").ignored(),
    ))).collect().map(move |patterns| PatternSet {patterns})
}

pub fn pattern_set_non_empty<'s>(
) -> impl Parser<'s, &'s str, PatternSet<'s>, extra::Err<Rich<'s, char>>> {
	single_pattern().separated_by(just(';').padded().recover_with(skip_then_retry_until(
        any().ignored(),
        one_of(";").ignored(),
    ))).at_least(1).collect().map(move |patterns| PatternSet {patterns})
}


pub fn single_pattern<'s>() -> impl Parser<'s, &'s str, Pattern<'s>, extra::Err<Rich<'s, char>>> {
    let (single_pat, mut expr_decl) = decl_single_pattern();
    let (single_expr, mut pat_decl) = decl_single_expression();

    expr_decl.define(single_expr.clone());
    pat_decl.define(single_pat.clone());

    single_pat.map(move |p|p.clone())
}

pub(crate) fn decl_single_pattern<'s>() -> ExpressionParserDelc<'s,'s> {
    let expression_declaration = Recursive::declare();

    let pattern_declartion = recursive(|pattern| {
        let literal = single_literal()
            .boxed()
            .map_with(move |l, meta| {
                Pattern::new_with_location(PatternBody::Literal(l), meta_to_location(meta))
            })
            .labelled("literal")
            .as_context()
            .boxed();

        let discard = just("_")
            .ignore_then(
                just("as")
                    .padded()
                    .ignore_then(single_type_literal())
                    .or_not(),
            )
            .map_with(move |value_type, meta| {
                if let Some(t) = value_type {
                    Pattern::new_with_location(PatternBody::TypedDiscard(t), meta_to_location(meta))
                } else {
                    Pattern::new_with_location(PatternBody::Discard, meta_to_location(meta))
                }
            })
            .boxed();

        let capture = single_identifier()
            .then(just("@").ignore_then(pattern.clone()).or_not())
            .boxed()
            .map_with(move |(id, pat), meta| {
                if let Some(p) = pat {
                    Pattern::new_with_location(
                        PatternBody::Capture(id, Box::new(p)),
                        meta_to_location(meta),
                    )
                } else {
                    Pattern::new_with_location(PatternBody::Identifier(id), meta_to_location(meta))
                }
            })
            .labelled("identifier")
            .as_context()
            .boxed();

        let typed_identifier = single_identifier()
            .then(just("as").padded().ignore_then(single_type_literal()))
            .map_with(move |(id, value_type), meta| {
                Pattern::new_with_location(
                    PatternBody::TypedIdentifier(id, value_type),
                    meta_to_location(meta),
                )
            })
            .boxed();

        let pinned = just("^")
            .ignore_then(choice((
                single_identifier().map_with(move |id, meta| {
                    Expression::new_with_location(
                        ExpressionBody::Identifier(id),
                        meta_to_location(meta),
                    )
                }),
                expression_declaration.clone().delimited_by(
                    just("("),
                    just(")")
                        .ignored()
                        .recover_with(via_parser(end()))
                        .recover_with(skip_then_retry_until(any().ignored(), end())),
                ),
            )))
            .map_with(move |expr, meta| {
                Pattern::new_with_location(
                    PatternBody::PinnedExpression(Box::new(expr)),
                    meta_to_location(meta),
                )
            })
            .boxed();

        let array = pattern
            .clone()
            .map(ArrayPatternItem::Pattern)
            .separated_by(just(',').padded().recover_with(skip_then_retry_until(
                any().ignored(),
                choice((just("]"), just(","))).ignored(),
            )))
            .allow_trailing()
            .collect()
            .then(
                just("...")
                    .padded()
                    .ignore_then(
                        pattern
                            .clone()
                            .map(Box::new)
                            .map(Rest::Collect)
                            .or_not()
                            .map(move |s| s.unwrap_or(Rest::Discard)),
                    )
                    .or_not(),
            )
            .padded()
            .delimited_by(
                just('['),
                just(']')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            )
            .map_with(move |(elements, rest), meta| {
                Pattern::new_with_location(
                    PatternBody::Array(elements, rest.unwrap_or(Rest::Exact)),
                    meta_to_location(meta),
                )
            })
            .labelled("array")
            .as_context()
            .boxed();

        let member = choice((
            single_string_literal()
                .map(move |name| Identifier { name })
                .map(PropertyKey::Identifier),
            single_identifier().map(PropertyKey::Identifier),
            expression_declaration
                .clone()
                .padded()
                .delimited_by(just("["), just("]"))
                .map(PropertyKey::Expression),
        ))
        .labelled("object_key")
        .as_context()
        .then_ignore(just(':').padded())
        .then(pattern.clone().labelled("value").as_context())
        .map(move |(key, value)| PropertyPattern { key, value })
        .map(ObjectPropertyPattern::Match)
        .or(single_identifier().map(ObjectPropertyPattern::Single))
        .boxed();

        let object = member
            .clone()
            .separated_by(just(',').padded().recover_with(skip_then_retry_until(
                any().ignored(),
                choice((just("}"), just(","))).ignored(),
            )))
            .allow_trailing()
            .collect()
            .then(
                just("...")
                    .padded()
                    .ignore_then(
                        pattern
                            .clone()
                            .map(Box::new)
                            .map(Rest::Collect)
                            .or_not()
                            .map(move |s| s.unwrap_or(Rest::Discard)),
                    )
                    .or_not(),
            )
            .padded()
            .delimited_by(
                just('{'),
                just('}')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            )
            .map_with(move |(entries, rest), meta| {
                Pattern::new_with_location(
                    PatternBody::Object(entries, rest.unwrap_or(Rest::Exact)),
                    meta_to_location(meta),
                )
            })
            .labelled("object")
            .as_context()
            .boxed();

        choice((
            literal,
            capture,
            discard,
            typed_identifier,
            pinned,
            array,
            object,
        ))
    })
    .padded()
    .boxed();

    (pattern_declartion, expression_declaration)
}
