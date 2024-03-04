use damasc_lang::syntax::expression::ExpressionSet;
use crate::pattern::decl_single_pattern;
use chumsky::recursive::Indirect;
use damasc_lang::identifier::Identifier;
use damasc_lang::literal::Literal;
use damasc_lang::syntax::expression::ArrayComprehension;
use damasc_lang::syntax::expression::BinaryExpression;
use damasc_lang::syntax::expression::BinaryOperator;
use damasc_lang::syntax::expression::CallExpression;
use damasc_lang::syntax::expression::ComprehensionSource;
use damasc_lang::syntax::expression::LambdaApplication;
use damasc_lang::syntax::expression::LogicalExpression;
use damasc_lang::syntax::expression::LogicalOperator;
use damasc_lang::syntax::expression::MemberExpression;
use damasc_lang::syntax::expression::ObjectComprehension;
use damasc_lang::syntax::expression::ObjectProperty;
use damasc_lang::syntax::expression::PropertyKey;
use damasc_lang::syntax::expression::StringTemplate;
use damasc_lang::syntax::expression::StringTemplatePart;
use damasc_lang::syntax::expression::UnaryExpression;
use damasc_lang::syntax::expression::UnaryOperator;
use damasc_lang::syntax::pattern::Pattern;
use damasc_lang::syntax::pattern::PatternBody;
use std::borrow::Cow;

use crate::identifier::single_identifier;
use crate::literal::single_string_literal;
use crate::util::meta_to_location;
use damasc_lang::syntax::expression::ArrayItem;
use damasc_lang::syntax::expression::Expression;
use damasc_lang::syntax::expression::ExpressionBody;
use damasc_lang::syntax::expression::IfElseExpression;
use damasc_lang::syntax::expression::LambdaAbstraction;
use damasc_lang::syntax::expression::MatchCase;
use damasc_lang::syntax::expression::MatchExpression;
use damasc_lang::syntax::expression::Property;

use crate::literal::single_literal;

use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

use chumsky::prelude::*;

type ExpressionParserDelc<'s,'a,'b> = (
    Boxed<'s, 's, &'s str, Expression<'a>, extra::Err<Rich<'s, char>>>,
    Recursive<Indirect<'s, 's, &'s str, Pattern<'b>, extra::Err<Rich<'s, char>>>>,
);

pub fn expression_set<'s,'x>(
) -> impl Parser<'s, &'s str, ExpressionSet<'x>, extra::Err<Rich<'s, char>>> {
	single_expression().map(|e| e.deep_clone()).separated_by(just(";").padded().recover_with(skip_then_retry_until(
        any().ignored(),
        choice((just(";"), just("with"))).padded().ignored(),
    ))).collect().map(|expressions| ExpressionSet {expressions})
}

pub fn expression_set_non_empty<'s,'x>(
) -> impl Parser<'s, &'s str, ExpressionSet<'x>, extra::Err<Rich<'s, char>>> {
	single_expression().map(|e| e.deep_clone()).separated_by(just(";").padded().recover_with(skip_then_retry_until(
        any().ignored(),
        choice((just(";"), just("with"))).padded().ignored(),
    ))).at_least(1).collect().map(|expressions| ExpressionSet {expressions})
}

pub fn single_expression<'s,'x>(
) -> impl Parser<'s, &'s str, Expression<'x>, extra::Err<Rich<'s, char>>> {
    let (single_pat, mut expr_decl) = decl_single_pattern();
    let (single_expr, mut pat_decl) = decl_single_expression();

    expr_decl.define(single_expr.clone());
    pat_decl.define(single_pat.clone());

    single_expr.map(|e| e.deep_clone())
}

pub(crate) fn decl_single_expression<'s>() -> ExpressionParserDelc<'s, 's,'s> {
    let pattern_declaration = Recursive::declare();

    let expression_declaration = recursive(|expression| {
        let boxed_expression = expression.clone().map(Box::new);

        let literal = single_literal()
            .map_with(|l, meta| {
                Expression::new_with_location(ExpressionBody::Literal(l), meta_to_location(meta))
            })
            .boxed();

        let matching_case = pattern_declaration
            .clone()
            .recover_with(skip_then_retry_until(
                any().ignored(),
                choice((just("if").padded(), just("=>").padded())).ignored(),
            ))
            .labelled("case_pattern")
            .as_context()
            .then(
                just("if")
                    .padded()
                    .ignore_then(boxed_expression.clone())
                    .or_not(),
            )
            .then_ignore(just("=>").padded())
            .then(boxed_expression.clone())
            .map(|((pattern, guard), body)| MatchCase {
                pattern,
                body,
                guard,
            })
            .boxed();

        let matching = just("match")
            .padded()
            .ignore_then(boxed_expression.clone())
            .then(
                matching_case
                    .clone()
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
                    .as_context(),
            )
            .map_with(|(subject, cases), meta| {
                Expression::new_with_location(
                    ExpressionBody::Match(MatchExpression { subject, cases }),
                    meta_to_location(meta),
                )
            })
            .boxed();

        let condition = just("if")
            .padded()
            .ignore_then(
                boxed_expression
                    .clone()
                    .recover_with(skip_then_retry_until(
                        any().ignored(),
                        choice((just("{").padded(),)).ignored(),
                    ))
                    .labelled("if_condition"),
            )
            .then(
                boxed_expression
                    .clone()
                    .delimited_by(
                        just('{').padded(),
                        just('}')
                            .padded()
                            .ignored()
                            .recover_with(via_parser(end()))
                            .recover_with(skip_then_retry_until(any().ignored(), end())),
                    )
                    .labelled("if_body")
                    .as_context(),
            )
            .then(
                just("else")
                    .padded()
                    .ignore_then(
                        boxed_expression
                            .clone()
                            .labelled("else_body")
                            .as_context()
                            .delimited_by(
                                just('{'),
                                just('}')
                                    .ignored()
                                    .recover_with(via_parser(end()))
                                    .recover_with(skip_then_retry_until(any().ignored(), end())),
                            ),
                    )
                    .or_not(),
            )
            .map_with(|((condition, true_branch), false_branch), meta| {
                Expression::new_with_location(
                    ExpressionBody::Condition(IfElseExpression {
                        condition,
                        true_branch,
                        false_branch,
                    }),
                    meta_to_location(meta),
                )
            })
            .boxed();

        let abstraction_param = pattern_declaration
            .clone()
            .delimited_by(
                just("("),
                just(")")
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            )
            .or(pattern_declaration.clone());

        let abstraction = just("fn")
            .padded()
            .ignore_then(
                abstraction_param
                    .clone()
                    .recover_with(skip_then_retry_until(
                        any().ignored(),
                        choice((just("=>").padded(),)).ignored(),
                    )),
            )
            .then(
                just("=>")
                    .padded()
                    .ignore_then(boxed_expression.clone().labelled("lamba_body").as_context()),
            )
            .map_with(|(arguments, body), meta| {
                Expression::new_with_location(
                    ExpressionBody::Abstraction(LambdaAbstraction { arguments, body }),
                    meta_to_location(meta),
                )
            })
            .labelled("lambda")
            .as_context()
            .boxed();

        let matching_abstraction = just("fn")
            .padded()
            .then(just("match").padded())
            .ignore_then(
                matching_case
                    .clone()
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
                    .as_context(),
            )
            .map_with(|cases, meta| {
                let local_identifier = Identifier::new("___local");
                ExpressionBody::Abstraction(LambdaAbstraction {
                    arguments: Pattern::new(PatternBody::Identifier(local_identifier.clone())),
                    body: Box::new(Expression::new_with_location(
                        ExpressionBody::Match(MatchExpression {
                            subject: Box::new(Expression::new(ExpressionBody::Identifier(
                                local_identifier,
                            ))),
                            cases,
                        }),
                        meta_to_location(meta),
                    )),
                })
            })
            .map_with(|body, meta| Expression::new_with_location(body, meta_to_location(meta)))
            .boxed();

        let expression_comprehension_source = just("for")
            .padded()
            .ignore_then(
                just("match")
                    .padded()
                    .or_not()
                    .map(|ref o| Option::is_none(o))
                    .padded(),
            )
            .then(
                pattern_declaration
                    .clone()
                    .recover_with(skip_then_retry_until(
                        any().ignored(),
                        choice((just("in").padded(),)).ignored(),
                    ))
                    .labelled("pattern")
                    .as_context(),
            )
            .then_ignore(just("in").padded())
            .then(
                boxed_expression
                    .clone()
                    .recover_with(skip_then_retry_until(
                        any().ignored(),
                        choice((just("if").padded(), just("]").padded())).ignored(),
                    ))
                    .labelled("expression")
                    .as_context(),
            )
            .then(
                just("if")
                    .padded()
                    .ignore_then(boxed_expression.clone().labelled("guard").as_context())
                    .or_not(),
            )
            .map(
                |(((strong_pattern, pattern), collection), predicate)| ComprehensionSource {
                    collection,
                    pattern,
                    strong_pattern,
                    predicate,
                },
            )
            .labelled("comprehension")
            .as_context()
            .boxed();

        let array = choice((
            expression.clone().map(ArrayItem::Single),
            just("...")
                .ignore_then(expression.clone())
                .map(ArrayItem::Spread),
        ))
        .separated_by(just(',').padded().recover_with(skip_then_retry_until(
            any().ignored(),
            choice((just("for"), just("]"), just(","))).ignored(),
        )))
        .allow_trailing()
        .collect()
        .padded()
        .then(
            expression_comprehension_source
                .clone()
                .repeated()
                .at_least(1)
                .collect()
                .or_not(),
        )
        .delimited_by(
            just('['),
            just(']')
                .ignored()
                .recover_with(via_parser(end()))
                .recover_with(skip_then_retry_until(any().ignored(), end())),
        )
        .map_with(|(elements, comprehension), meta| {
            if let Some(comp) = comprehension {
                Expression::new_with_location(
                    ExpressionBody::ArrayComp(ArrayComprehension {
                        projection: elements,
                        sources: comp,
                    }),
                    meta_to_location(meta),
                )
            } else {
                Expression::new_with_location(
                    ExpressionBody::Array(elements),
                    meta_to_location(meta),
                )
            }
        })
        .labelled("array")
        .as_context()
        .boxed();

        let member = choice((
            single_string_literal()
                .map(|name| Identifier { name })
                .map(PropertyKey::Identifier),
            single_identifier().map(PropertyKey::Identifier),
            expression
                .clone()
                .padded()
                .delimited_by(just("["), just("]"))
                .map(PropertyKey::Expression),
        ))
        .labelled("object_key")
        .as_context()
        .then_ignore(just(':').padded())
        .then(expression.clone().labelled("value").as_context())
        .map(|(key, value)| Property { key, value })
        .map(ObjectProperty::Property)
        .or(single_identifier().map(ObjectProperty::Single))
        .or(just("...")
            .ignore_then(expression.clone())
            .map(ObjectProperty::Spread))
        .boxed();

        let object = member
            .clone()
            .separated_by(just(',').padded().recover_with(skip_then_retry_until(
                any().ignored(),
                choice((just("for"), just("}"), just(","))).ignored(),
            )))
            .allow_trailing()
            .collect()
            .then(
                expression_comprehension_source
                    .clone()
                    .repeated()
                    .at_least(1)
                    .collect()
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
            .map_with(|(entries, comprehension), meta| {
                if let Some(comp) = comprehension {
                    Expression::new_with_location(
                        ExpressionBody::ObjectComp(ObjectComprehension {
                            projection: entries,
                            sources: comp,
                        }),
                        meta_to_location(meta),
                    )
                } else {
                    Expression::new_with_location(
                        ExpressionBody::Object(entries),
                        meta_to_location(meta),
                    )
                }
            })
            .labelled("object")
            .as_context()
            .boxed();

        let string_template_part_static = (none_of("$")
            .to_slice()
            .or(just("$").then(none_of("{")).to_slice()))
        .repeated()
        .to_slice();
        let string_template_part_dynamic = boxed_expression.clone().delimited_by(
            just("${"),
            just("}")
                .ignored()
                .recover_with(via_parser(end()))
                .recover_with(skip_then_retry_until(any().ignored(), end())),
        );
        let string_template_part = string_template_part_static
            .then(string_template_part_dynamic)
            .map(|(fixed_start, dynamic_end)| StringTemplatePart {
                fixed_start: Cow::Owned(fixed_start.to_string()),
                dynamic_end,
            });

        let string_template = string_template_part
            .repeated()
            .collect()
            .then(none_of("`").repeated().to_slice().map(Cow::Borrowed))
            .delimited_by(just("`"), just("`"))
            .map(|(parts, suffix)| ExpressionBody::Template(StringTemplate { parts, suffix }))
            .map_with(|body, meta| Expression::new_with_location(body, meta_to_location(meta)));

        let ident = single_identifier()
            .map(ExpressionBody::Identifier)
            .map_with(|body, meta| Expression::new_with_location(body, meta_to_location(meta)))
            .boxed();

        let parenthesis = expression.clone().delimited_by(
            just('('),
            just(')')
                .ignored()
                .recover_with(via_parser(end()))
                .recover_with(skip_then_retry_until(any().ignored(), end())),
        );

        enum PathSegment<'a> {
            Application(Expression<'a>),
            Index(Expression<'a>),
            Prop(Expression<'a>),
        }

        let path_indexed = expression
            .clone()
            .delimited_by(
                just('['),
                just(']')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            )
            .map(PathSegment::Index)
            .boxed();

        let path_property = just(".")
            .ignore_then(single_identifier())
            .map_with(|ident, meta| {
                Expression::new_with_location(
                    ExpressionBody::Literal(Literal::String(ident.name)),
                    meta_to_location(meta),
                )
            })
            .map(PathSegment::Prop)
            .boxed();
        let path_apply = just(".")
            .ignore_then(parenthesis.clone())
            .map(PathSegment::Application)
            .boxed();

        let call = single_identifier()
            .then(parenthesis.clone())
            .map_with(|(function, argument), meta| {
                Expression::new_with_location(
                    ExpressionBody::Call(CallExpression {
                        function,
                        argument: Box::new(argument),
                    }),
                    meta_to_location(meta),
                )
            })
            .boxed();

        let path_base = choice((
            matching_abstraction,
            abstraction,
            matching,
            condition,
            literal,
            array,
            object,
            string_template,
            call,
            ident,
            parenthesis,
        ))
        .boxed();

        let path = path_base.clone().foldl_with(
            choice((path_property, path_apply, path_indexed)).repeated(),
            |expr, segment, meta| {
                Expression::new_with_location(
                    match segment {
                        PathSegment::Application(param) => {
                            ExpressionBody::Application(LambdaApplication {
                                lambda: Box::new(expr),
                                parameter: Box::new(param),
                            })
                        }
                        PathSegment::Index(index) => ExpressionBody::Member(MemberExpression {
                            object: Box::new(expr),
                            property: Box::new(index),
                        }),
                        PathSegment::Prop(property) => ExpressionBody::Member(MemberExpression {
                            object: Box::new(expr),
                            property: Box::new(property),
                        }),
                    },
                    meta_to_location(meta),
                )
            },
        );

        // TODO: Prefer Pratt-Parser

        let unary_operator = choice((
            just("!").padded().map(|_| UnaryOperator::Not),
            just("+").padded().map(|_| UnaryOperator::Plus),
            just("-").padded().map(|_| UnaryOperator::Minus),
        ));

        let unary_op = unary_operator
            .then(boxed_expression.clone().labelled("operand").as_context())
            .map_with(|(operator, argument), meta| {
                Expression::new_with_location(
                    ExpressionBody::Unary(UnaryExpression { operator, argument }),
                    meta_to_location(meta),
                )
            })
            .boxed();

        let binary_atom = choice((unary_op, path)).boxed();

        let num_exponential = binary_atom
            .clone()
            .foldl_with(
                choice((just("^").padded().map(|_| BinaryOperator::PowerOf),))
                    .then(binary_atom.clone().labelled("operand").as_context())
                    .repeated(),
                |lhs, (operator, rhs), meta| {
                    Expression::new_with_location(
                        ExpressionBody::Binary(BinaryExpression {
                            operator,
                            left: Box::new(lhs),
                            right: Box::new(rhs),
                        }),
                        meta_to_location(meta),
                    )
                },
            )
            .boxed();

        let num_mul = num_exponential
            .clone()
            .foldl_with(
                choice((
                    just("*").padded().map(|_| BinaryOperator::Times),
                    just("/").padded().map(|_| BinaryOperator::Over),
                ))
                .then(num_exponential.clone().labelled("operand").as_context())
                .repeated(),
                |lhs, (operator, rhs), meta| {
                    Expression::new_with_location(
                        ExpressionBody::Binary(BinaryExpression {
                            operator,
                            left: Box::new(lhs),
                            right: Box::new(rhs),
                        }),
                        meta_to_location(meta),
                    )
                },
            )
            .boxed();

        let num_add = num_mul
            .clone()
            .foldl_with(
                choice((
                    just("+").padded().map(|_| BinaryOperator::Plus),
                    just("-").padded().map(|_| BinaryOperator::Minus),
                ))
                .then(num_mul.clone().labelled("operand").as_context())
                .repeated(),
                |lhs, (operator, rhs), meta| {
                    Expression::new_with_location(
                        ExpressionBody::Binary(BinaryExpression {
                            operator,
                            left: Box::new(lhs),
                            right: Box::new(rhs),
                        }),
                        meta_to_location(meta),
                    )
                },
            )
            .boxed();

        let num_pred = num_add
            .clone()
            .foldl_with(
                choice((
                    just(">=")
                        .padded()
                        .map(|_| BinaryOperator::GreaterThanEqual),
                    just("<=").padded().map(|_| BinaryOperator::LessThanEqual),
                    just("==").padded().map(|_| BinaryOperator::StrictEqual),
                    just("!=").padded().map(|_| BinaryOperator::StrictNotEqual),
                    just(">").padded().map(|_| BinaryOperator::GreaterThan),
                    just("<").padded().map(|_| BinaryOperator::LessThan),
                    just("in").padded().map(|_| BinaryOperator::In),
                ))
                .then(num_add.clone().labelled("operand").as_context())
                .repeated(),
                |lhs, (operator, rhs), meta| {
                    Expression::new_with_location(
                        ExpressionBody::Binary(BinaryExpression {
                            operator,
                            left: Box::new(lhs),
                            right: Box::new(rhs),
                        }),
                        meta_to_location(meta),
                    )
                },
            )
            .boxed();

        let type_add = num_pred
            .clone()
            .foldl_with(
                choice((just("as").padded().map(|_| BinaryOperator::Cast),))
                    .then(num_pred.clone().labelled("operand").as_context())
                    .repeated(),
                |lhs, (operator, rhs), meta| {
                    Expression::new_with_location(
                        ExpressionBody::Binary(BinaryExpression {
                            operator,
                            left: Box::new(lhs),
                            right: Box::new(rhs),
                        }),
                        meta_to_location(meta),
                    )
                },
            )
            .boxed();

        let type_pred = type_add
            .clone()
            .foldl_with(
                choice((just("is").padded().map(|_| BinaryOperator::Is),))
                    .then(type_add.clone().labelled("operand").as_context())
                    .repeated(),
                |lhs, (operator, rhs), meta| {
                    Expression::new_with_location(
                        ExpressionBody::Binary(BinaryExpression {
                            operator,
                            left: Box::new(lhs),
                            right: Box::new(rhs),
                        }),
                        meta_to_location(meta),
                    )
                },
            )
            .boxed();

        let logic_mul = type_pred
            .clone()
            .foldl_with(
                choice((just("&&").padded().map(|_| LogicalOperator::And),))
                    .then(type_pred.clone().labelled("operand").as_context())
                    .repeated(),
                |lhs, (operator, rhs), meta| {
                    Expression::new_with_location(
                        ExpressionBody::Logical(LogicalExpression {
                            operator,
                            left: Box::new(lhs),
                            right: Box::new(rhs),
                        }),
                        meta_to_location(meta),
                    )
                },
            )
            .boxed();

        let logic_add = logic_mul
            .clone()
            .foldl_with(
                choice((just("||").padded().map(|_| LogicalOperator::Or),))
                    .then(logic_mul.clone().labelled("operand").as_context())
                    .repeated(),
                |lhs, (operator, rhs), meta| {
                    Expression::new_with_location(
                        ExpressionBody::Logical(LogicalExpression {
                            operator,
                            left: Box::new(lhs),
                            right: Box::new(rhs),
                        }),
                        meta_to_location(meta),
                    )
                },
            )
            .boxed();

        logic_add.padded()
    }).boxed();

    (expression_declaration, pattern_declaration)
}
