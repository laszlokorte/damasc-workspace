
use damasc_lang::syntax::expression::LogicalOperator;
use damasc_lang::syntax::expression::LogicalExpression;
use damasc_lang::syntax::expression::BinaryOperator;
use damasc_lang::syntax::expression::BinaryExpression;
use damasc_lang::syntax::expression::UnaryExpression;
use damasc_lang::syntax::expression::UnaryOperator;
use damasc_lang::syntax::pattern::Pattern;
use chumsky::recursive::Indirect;
use damasc_lang::syntax::expression::ObjectComprehension;
use damasc_lang::syntax::expression::ComprehensionSource;
use damasc_lang::syntax::expression::ArrayComprehension;
use damasc_lang::literal::Literal;
use damasc_lang::syntax::expression::LambdaApplication;
use damasc_lang::syntax::expression::MemberExpression;
use damasc_lang::syntax::expression::StringTemplate;
use damasc_lang::syntax::expression::StringTemplatePart;
use std::borrow::Cow;
use damasc_lang::syntax::expression::ObjectProperty;
use damasc_lang::identifier::Identifier;
use damasc_lang::syntax::expression::PropertyKey;

use damasc_lang::syntax::expression::Property;
use damasc_lang::syntax::expression::ArrayItem;
use crate::literal::single_string_literal;
use crate::identifier::single_identifier;
use damasc_lang::syntax::expression::LambdaAbstraction;
use damasc_lang::syntax::expression::IfElseExpression;
use damasc_lang::syntax::expression::MatchCase;
use damasc_lang::syntax::expression::MatchExpression;
use crate::util::meta_to_location;
use damasc_lang::syntax::expression::ExpressionBody;
use damasc_lang::syntax::expression::Expression;

use crate::literal::single_literal;

use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

use chumsky::prelude::*;

pub fn single_expression<'s>() -> (Boxed<'s, 's, &'s str, Expression<'s>, extra::Err<Rich<'s, char>>>, Recursive<Indirect<'s, 's, &'s str, Pattern<'s>, extra::Err<Rich<'s, char>>>>)  {

    let pattern_declaration = Recursive::declare();

    let expression_declaration = recursive(|expression| {
        let boxed_expression = expression.clone().map(Box::new);

        let literal = single_literal().map_with(|l, meta| Expression::new_with_location(ExpressionBody::Literal(l), meta_to_location(meta))).boxed();

        let matching_case = pattern_declaration.clone()
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
        .ignore_then(pattern_declaration.clone())
        .then(just("=>").padded().ignore_then(boxed_expression.clone()))
        .map_with(|(arguments, body), meta| Expression::new_with_location(ExpressionBody::Abstraction(LambdaAbstraction {
                    arguments,
                    body,
                }), meta_to_location(meta))).boxed();



        let expression_comprehension_source = just("for").padded()
        .ignore_then(just("match").padded().or_not().map(|ref o| Option::is_none(o)).padded())
        .then(pattern_declaration.clone()
            .labelled("pattern")
            .as_context())
        .then_ignore(just("in"))
        .then(boxed_expression.clone()
            .labelled("expression")
            .as_context())
        .then(just("if").padded().ignore_then(boxed_expression.clone()
            .labelled("guard")
            .as_context()).or_not())
        .map(|(((strong_pattern, pattern), collection), predicate)| {
            ComprehensionSource {  
                collection,
                pattern,
                strong_pattern,
                predicate,
            }
        })
            .labelled("comprehension")
            .as_context().boxed();

        let array = choice((expression.clone().map(ArrayItem::Single), just("...").ignore_then(expression.clone()).map(ArrayItem::Spread)))
            .separated_by(just(',').padded().recover_with(skip_then_retry_until(
                any().ignored(),
                choice((just("for"), just("]"), just(","))).ignored(),
            )))
            .allow_trailing()
            .collect()
            .padded()
            .then(expression_comprehension_source.clone().repeated().at_least(1).collect().or_not())
            .delimited_by(
                just('['),
                just(']')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            )
            .map_with(|(elements, comprehension), meta| {
                if let Some(comp) = comprehension {
                    Expression::new_with_location(ExpressionBody::ArrayComp(ArrayComprehension{projection: elements, sources: comp}), meta_to_location(meta))
                } else {
                    Expression::new_with_location(ExpressionBody::Array(elements), meta_to_location(meta))
                }
            })
            .labelled("array")
            .as_context()
            .boxed();

        let member = 
            choice((
                single_string_literal().map(|name|Identifier{name}).map(PropertyKey::Identifier),
                single_identifier().map(PropertyKey::Identifier),
                expression.clone().padded().delimited_by(just("["), just("]")).map(PropertyKey::Expression),
            ))
            .labelled("object_key")
            .as_context()
            .then_ignore(just(':').padded())
            .then(expression.clone().labelled("value").as_context())
            .map(|(key,value)| Property{key,value})
            .map(ObjectProperty::Property)
            .or(single_identifier().map(ObjectProperty::Single))
            .or(just("...").ignore_then(expression.clone()).map(ObjectProperty::Spread))
            .boxed();

        let object = member
            .clone()
            .separated_by(just(',').padded().recover_with(skip_then_retry_until(
                any().ignored(),
                choice((just("for"), just("]"), just(","))).ignored(),
            )))
            .allow_trailing()
            .collect()
            .then(expression_comprehension_source.clone().repeated().at_least(1).collect().or_not())
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
                    Expression::new_with_location(ExpressionBody::ObjectComp(ObjectComprehension{projection: entries, sources: comp}), meta_to_location(meta))
                } else {
                    Expression::new_with_location(ExpressionBody::Object(entries), meta_to_location(meta))
                }
            })
            .labelled("object")
            .as_context()
            .boxed();


        let string_template_part_static = (none_of("$").to_slice().or(just("$").then(none_of("{")).to_slice())).repeated().to_slice();
        let string_template_part_dynamic = boxed_expression.clone().delimited_by(just("${"), just("}")
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())));
        let string_template_part = string_template_part_static.then(string_template_part_dynamic).map(|(fixed_start, dynamic_end)| {
            StringTemplatePart {
                fixed_start: Cow::Borrowed(fixed_start),
                dynamic_end,
            }
        });

        let string_template = string_template_part.repeated().collect().then(none_of("`").repeated().to_slice().map(Cow::Borrowed)).delimited_by(just("`"), just("`"))
        .map(|(parts, suffix)| {
            ExpressionBody::Template(StringTemplate {
                parts,
                suffix,
            })
        }).map_with(|body, meta| {
            Expression::new_with_location(body, meta_to_location(meta))
        });

        let ident = single_identifier().map(ExpressionBody::Identifier).map_with(|body, meta| {
            Expression::new_with_location(body, meta_to_location(meta))
        }).boxed();

        let parenthesis = expression.clone().delimited_by(
                just('('),
                just(')')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),);




        enum PathSegment<'a> {
            Application(Expression<'a>),
            Index(Expression<'a>),
            Prop(Expression<'a>),
        }

        let path_indexed = expression.clone().delimited_by(
                just('['),
                just(']')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),)
        .map(PathSegment::Index).boxed();

        let path_property = just(".").ignore_then(single_identifier())
        .map_with(|ident, meta| {
            Expression::new_with_location(
                ExpressionBody::Literal(Literal::String(ident.name)),
                meta_to_location(meta),
            )
        })
        .map(PathSegment::Prop).boxed();
        let path_apply = just(".").ignore_then(parenthesis.clone())
        .map(PathSegment::Application).boxed();

        let path_base = choice((
            abstraction, matching, condition, literal, array, object, string_template, ident, parenthesis
        )).boxed();

        let path = path_base.clone().foldl_with(choice((
            path_property,
            path_apply,
            path_indexed,
        )).repeated(), |expr, segment, meta| {
            Expression::new_with_location(match segment {
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
            }, meta_to_location(meta))
        });

        // TODO: Prefer Pratt-Parser

        let unary_operator = choice((
            just("!").padded().map(|_| UnaryOperator::Not,),
            just("+").padded().map(|_| UnaryOperator::Plus,),
            just("-").padded().map(|_| UnaryOperator::Minus,),
        ));

        let unary_op = unary_operator.then(boxed_expression.clone()).map_with(|(operator, argument), meta| Expression::new_with_location(ExpressionBody::Unary(UnaryExpression{operator, argument}), meta_to_location(meta))).boxed();
    
        let binary_atom = choice((unary_op, path)).boxed();

        let num_exponential = binary_atom.clone().foldl_with(choice((
            just("^").padded().map(|_| BinaryOperator::PowerOf,),
        )).then(binary_atom.clone()).repeated(), |lhs, (operator, rhs), meta| {
            Expression::new_with_location(ExpressionBody::Binary(BinaryExpression {
                operator,
                left: Box::new(lhs),
                right: Box::new(rhs),
            }), meta_to_location(meta))
        }).boxed();

        let num_mul = num_exponential.clone().foldl_with(choice((
            just("*").padded().map(|_| BinaryOperator::Times,),
            just("/").padded().map(|_| BinaryOperator::Over,),
        )).then(num_exponential.clone()).repeated(), |lhs, (operator, rhs), meta| {
            Expression::new_with_location(ExpressionBody::Binary(BinaryExpression {
                operator,
                left: Box::new(lhs),
                right: Box::new(rhs),
            }), meta_to_location(meta))
        }).boxed();

        let num_add = num_mul.clone().foldl_with(choice((
            just("+").padded().map(|_| BinaryOperator::Plus,),
            just("-").padded().map(|_| BinaryOperator::Minus,),
        )).then(num_mul.clone()).repeated(), |lhs, (operator, rhs), meta| {
            Expression::new_with_location(ExpressionBody::Binary(BinaryExpression {
                operator,
                left: Box::new(lhs),
                right: Box::new(rhs),
            }), meta_to_location(meta))
        }).boxed();

        let num_pred = num_add.clone().foldl_with(choice((
            just(">").padded().map(|_| BinaryOperator::GreaterThan,),
            just("<").padded().map(|_| BinaryOperator::LessThan,),
            just(">=").padded().map(|_| BinaryOperator::GreaterThanEqual,),
            just("<=").padded().map(|_| BinaryOperator::LessThanEqual,),
            just("==").padded().map(|_| BinaryOperator::StrictEqual,),
            just("!=").padded().map(|_| BinaryOperator::StrictNotEqual,),
        )).then(num_add.clone()).repeated(), |lhs, (operator, rhs), meta| {
            Expression::new_with_location(ExpressionBody::Binary(BinaryExpression {
                operator,
                left: Box::new(lhs),
                right: Box::new(rhs),
            }), meta_to_location(meta))
        }).boxed();

        let type_add = num_pred.clone().foldl_with(choice((
            just("as").padded().map(|_| BinaryOperator::Cast,),
        )).then(num_pred.clone()).repeated(), |lhs, (operator, rhs), meta| {
            Expression::new_with_location(ExpressionBody::Binary(BinaryExpression {
                operator,
                left: Box::new(lhs),
                right: Box::new(rhs),
            }), meta_to_location(meta))
        }).boxed();

        let type_pred = type_add.clone().foldl_with(choice((
            just("is").padded().map(|_| BinaryOperator::Is,),
        )).then(type_add.clone()).repeated(), |lhs, (operator, rhs), meta| {
            Expression::new_with_location(ExpressionBody::Binary(BinaryExpression {
                operator,
                left: Box::new(lhs),
                right: Box::new(rhs),
            }), meta_to_location(meta))
        }).boxed();

        let logic_mul = type_pred.clone().foldl_with(choice((
            just("&&").padded().map(|_| LogicalOperator::And,),
        )).then(type_pred.clone()).repeated(), |lhs, (operator, rhs), meta| {
            Expression::new_with_location(ExpressionBody::Logical(LogicalExpression {
                operator,
                left: Box::new(lhs),
                right: Box::new(rhs),
            }), meta_to_location(meta))
        }).boxed();

        let logic_add = logic_mul.clone().foldl_with(choice((
            just("||").padded().map(|_| LogicalOperator::Or,),
        )).then(logic_mul.clone()).repeated(), |lhs, (operator, rhs), meta| {
            Expression::new_with_location(ExpressionBody::Logical(LogicalExpression {
                operator,
                left: Box::new(lhs),
                right: Box::new(rhs),
            }), meta_to_location(meta))
        }).boxed();

        logic_add.padded()
    }).boxed();

    (expression_declaration, pattern_declaration)
}
