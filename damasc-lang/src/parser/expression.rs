use crate::literal::Literal;
use crate::parser::located::located_expression;
use crate::parser::pattern::pattern;
use crate::syntax::expression::ArrayComprehension;
use crate::syntax::expression::ComprehensionSource;
use crate::syntax::expression::IfElseExpression;
use crate::syntax::expression::LambdaAbstraction;
use crate::syntax::expression::LambdaApplication;
use crate::syntax::expression::MatchCase;
use crate::syntax::expression::MatchExpression;
use crate::syntax::expression::ObjectComprehension;
use crate::syntax::location::Location;
use crate::syntax::pattern::Pattern;
use crate::syntax::pattern::PatternBody;
use nom::multi::many1;
use nom_locate::position;

use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{alpha1, space0},
    combinator::{all_consuming, map, not, opt, peek, recognize, value},
    error::{context, Error},
    multi::{fold_many0, many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

use crate::{
    identifier::Identifier,
    syntax::expression::{
        ArrayItem, BinaryExpression, BinaryOperator, CallExpression, Expression, ExpressionBody,
        ExpressionSet, LogicalExpression, LogicalOperator, MemberExpression, ObjectProperty,
        Property, PropertyKey, StringTemplate, StringTemplatePart, UnaryExpression, UnaryOperator,
    },
};

use super::{
    identifier::identifier,
    io::{ParserError, ParserInput, ParserResult},
    literal::{literal, literal_string_raw},
    util::ws,
};

pub fn expression_all_consuming<'v>(input: &str) -> Option<Expression<'v>> {
    match all_consuming(expression::<Error<ParserInput>>)(ParserInput::new(input)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

pub fn expression_many1<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<ExpressionSet<'v>, E> {
    context(
        "expression_many1",
        delimited(
            space0,
            map(separated_list1(ws(tag(";")), expression), |expressions| {
                ExpressionSet { expressions }
            }),
            opt(ws(tag(";"))),
        ),
    )(input)
}

pub fn expression_many1_all_consuming<'v>(input: &str) -> Option<ExpressionSet<'v>> {
    match all_consuming(expression_many1::<Error<ParserInput>>)(ParserInput::new(input)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

pub fn expression_many0_all_consuming<'v>(input: &str) -> Option<ExpressionSet<'v>> {
    match all_consuming(expression_many0::<Error<ParserInput>>)(ParserInput::new(input)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

pub fn expression_many0<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<ExpressionSet<'v>, E> {
    context(
        "expression_many0",
        delimited(
            space0,
            map(separated_list0(ws(tag(";")), expression), |expressions| {
                ExpressionSet { expressions }
            }),
            ws(opt(tag(";"))),
        ),
    )(input)
}

fn expression_array_item<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<ArrayItem<'v>, E> {
    context(
        "expression_array_item",
        alt((
            map(preceded(ws(tag("...")), expression), ArrayItem::Spread),
            map(expression, ArrayItem::Single),
        )),
    )(input)
}

fn expression_call<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_call",
        located_expression(map(
            pair(
                identifier,
                delimited(ws(tag("(")), expression, ws(tag(")"))),
            ),
            |(function, arg)| {
                ExpressionBody::Call(CallExpression {
                    function,
                    argument: Box::new(arg),
                })
            },
        )),
    )(input)
}

fn expression_array<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_array",
        located_expression(delimited(
            ws(tag("[")),
            map(
                opt(terminated(
                    map(
                        separated_list1(ws(tag(",")), expression_array_item),
                        ExpressionBody::Array,
                    ),
                    opt(ws(tag(","))),
                )),
                |v| v.unwrap_or_else(|| ExpressionBody::Array(vec![])),
            ),
            ws(tag("]")),
        )),
    )(input)
}

fn expression_array_comprehension<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_array_comprehension",
        located_expression(map(
            delimited(
                ws(tag("[")),
                tuple((
                    terminated(
                        separated_list1(ws(tag(",")), expression_array_item),
                        opt(ws(tag(","))),
                    ),
                    expression_comprehension_source,
                )),
                ws(tag("]")),
            ),
            |(projection, sources)| {
                ExpressionBody::ArrayComp(ArrayComprehension {
                    projection,
                    sources,
                })
            },
        )),
    )(input)
}

fn expression_object_property<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<ObjectProperty<'v>, E> {
    context(
        "expression_object_property",
        alt((
            map(
                separated_pair(
                    delimited(ws(tag("[")), expression, ws(tag("]"))),
                    ws(tag(":")),
                    expression,
                ),
                |(prop, value)| {
                    ObjectProperty::Property(Property {
                        key: PropertyKey::Expression(prop),
                        value,
                    })
                },
            ),
            map(
                separated_pair(identifier, ws(tag(":")), expression),
                |(prop, value)| {
                    ObjectProperty::Property(Property {
                        key: PropertyKey::Identifier(prop),
                        value,
                    })
                },
            ),
            map(
                separated_pair(literal_string_raw, ws(tag(":")), expression),
                |(prop, value)| {
                    ObjectProperty::Property(Property {
                        key: PropertyKey::Identifier(Identifier { name: prop }),
                        value,
                    })
                },
            ),
            map(preceded(ws(tag("...")), expression), ObjectProperty::Spread),
            map(identifier, ObjectProperty::Single),
        )),
    )(input)
}

fn expression_object<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_object",
        located_expression(delimited(
            ws(tag("{")),
            map(
                opt(terminated(
                    map(
                        separated_list1(ws(tag(",")), expression_object_property),
                        ExpressionBody::Object,
                    ),
                    opt(ws(tag(","))),
                )),
                |v| v.unwrap_or_else(|| ExpressionBody::Object(vec![])),
            ),
            ws(tag("}")),
        )),
    )(input)
}

fn expression_comprehension_source<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Vec<ComprehensionSource<'v>>, E> {
    context(
        "expression_comprehension_source",
        many1(map(
            tuple((
                preceded(ws(tag("for")), tuple((opt(ws(tag("match"))), pattern))),
                preceded(ws(tag("in")), expression),
                opt(preceded(ws(tag("if")), expression)),
            )),
            |((weak, pattern), collection, predicate)| ComprehensionSource {
                strong_pattern: weak.is_none(),
                pattern,
                collection: Box::new(collection),
                predicate: predicate.map(Box::new),
            },
        )),
    )(input)
}

fn expression_object_comprehension<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_object_comprehension",
        located_expression(map(
            delimited(
                ws(tag("{")),
                tuple((
                    terminated(
                        separated_list1(ws(tag(",")), expression_object_property),
                        opt(ws(tag(","))),
                    ),
                    expression_comprehension_source,
                )),
                ws(tag("}")),
            ),
            |(projection, sources)| {
                ExpressionBody::ObjectComp(ObjectComprehension {
                    projection,
                    sources,
                })
            },
        )),
    )(input)
}

fn expression_lambda_abstraction<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_lambda_abstraction",
        located_expression(map(
            tuple((
                preceded(
                    ws(tag("fn")),
                    alt((delimited(ws(tag("(")), pattern, ws(tag(")"))), pattern)),
                ),
                preceded(ws(tag("=>")), expression),
            )),
            |(arg, body)| {
                ExpressionBody::Abstraction(LambdaAbstraction {
                    arguments: arg,
                    body: Box::new(body),
                })
            },
        )),
    )(input)
}

fn expression_match_arm<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<MatchCase<'v>, E> {
    map(
        separated_pair(
            tuple((
                alt((delimited(ws(tag("(")), pattern, ws(tag(")"))), pattern)),
                opt(preceded(ws(tag("if")), expression)),
            )),
            ws(tag("=>")),
            expression,
        ),
        |((pattern, guard), body)| MatchCase {
            pattern,
            guard: guard.map(Box::new),
            body: Box::new(body),
        },
    )(input)
}

fn expression_match<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_match",
        located_expression(map(
            tuple((
                preceded(
                    ws(tag("match")),
                    alt((
                        delimited(ws(tag("(")), expression, ws(tag(")"))),
                        expression,
                    )),
                ),
                delimited(
                    ws(tag("{")),
                    opt(terminated(
                        separated_list1(ws(tag(",")), expression_match_arm),
                        opt(ws(tag(","))),
                    )),
                    ws(tag("}")),
                ),
            )),
            |(subject, cases)| {
                ExpressionBody::Match(MatchExpression {
                    subject: Box::new(subject),
                    cases: cases.unwrap_or_default(),
                })
            },
        )),
    )(input)
}

fn expression_condition<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_condition",
        located_expression(map(
            tuple((
                preceded(
                    ws(tag("if")),
                    alt((
                        delimited(ws(tag("(")), expression, ws(tag(")"))),
                        expression,
                    )),
                ),
                delimited(ws(tag("{")), expression, ws(tag("}"))),
                opt(preceded(
                    ws(tag("else")),
                    delimited(ws(tag("{")), expression, ws(tag("}"))),
                )),
            )),
            |(condition, true_branch, false_branch)| {
                ExpressionBody::Condition(IfElseExpression {
                    condition: Box::new(condition),
                    true_branch: Box::new(true_branch),
                    false_branch: false_branch.map(Box::new),
                })
            },
        )),
    )(input)
}

fn expression_lambda_match_abstraction<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_lambda_match_abstraction",
        located_expression(map(
            preceded(
                ws(tag("fn match")),
                delimited(
                    ws(tag("{")),
                    opt(terminated(
                        separated_list1(ws(tag(",")), expression_match_arm),
                        opt(ws(tag(","))),
                    )),
                    ws(tag("}")),
                ),
            ),
            |cases| {
                let local_identifier = Identifier::new("___local");
                ExpressionBody::Abstraction(LambdaAbstraction {
                    arguments: Pattern::new(PatternBody::Identifier(local_identifier.clone())),
                    body: Box::new(Expression::new(ExpressionBody::Match(MatchExpression {
                        subject: Box::new(Expression::new(ExpressionBody::Identifier(
                            local_identifier,
                        ))),
                        cases: cases.unwrap_or_default(),
                    }))),
                })
            },
        )),
    )(input)
}

fn expression_literal<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_literal",
        alt((
            expression_object,
            expression_array,
            expression_array_comprehension,
            expression_object_comprehension,
            expression_string_template,
            expression_call,
            expression_atom,
        )),
    )(input)
}

fn expression_atom<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_atom",
        located_expression(map(literal, ExpressionBody::Literal)),
    )(input)
}

pub(crate) fn expression_identifier<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_identifier",
        located_expression(map(identifier, ExpressionBody::Identifier)),
    )(input)
}

fn string_template_part<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<StringTemplatePart<'v>, E> {
    context(
        "expression_string_template_part",
        map(
            tuple((
                recognize(take_until("${")),
                delimited(
                    tag("${"),
                    context("expression_string_template_part_dynamic", expression),
                    tag("}"),
                ),
            )),
            |(fixed_start, dynamic_end)| StringTemplatePart {
                fixed_start: Cow::Owned(fixed_start.fragment().to_owned().into()),
                dynamic_end: Box::new(dynamic_end),
            },
        ),
    )(input)
}

fn expression_string_template<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_string_template",
        located_expression(map(
            delimited(
                tag("`"),
                tuple((many0(string_template_part), recognize(many0(is_not("`"))))),
                tag("`"),
            ),
            |(parts, s)| {
                ExpressionBody::Template(StringTemplate {
                    parts,
                    suffix: Cow::Owned(s.to_string()),
                })
            },
        )),
    )(input)
}

fn expression_logic_additive_operator<'s, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<LogicalOperator, E> {
    context(
        "expression_logic_operator",
        alt((value(LogicalOperator::Or, tag("||")),)),
    )(input)
}

fn expression_logic_additive<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    let (input, init) = context(
        "expression_logic_additive_lhs",
        expression_logic_multiplicative,
    )(input)?;

    fold_many0(
        pair(
            ws(expression_logic_additive_operator),
            context(
                "expression_logic_additive_rhs",
                expression_logic_multiplicative,
            ),
        ),
        move || init.clone(),
        |left, (operator, right)| {
            let outer_location = left
                .location
                .and_then(|l| right.location.map(|r| Location::new(l.start, r.end)));

            Expression::new_with_optional_location(
                ExpressionBody::Logical(LogicalExpression {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                outer_location,
            )
        },
    )(input)
}

fn expression_logic_multiplicative_operator<'s, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<LogicalOperator, E> {
    context(
        "expression_logic_operator",
        alt((value(LogicalOperator::And, tag("&&")),)),
    )(input)
}

fn expression_logic_multiplicative<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    let (input, init) = context(
        "expression_logic_multiplicative_lhs",
        expression_type_predicate,
    )(input)?;

    fold_many0(
        pair(
            ws(expression_logic_multiplicative_operator),
            context(
                "expression_logic_multiplicative_rhs",
                expression_type_predicate,
            ),
        ),
        move || init.clone(),
        |left, (operator, right)| {
            let outer_location = left
                .location
                .and_then(|l| right.location.map(|r| Location::new(l.start, r.end)));

            Expression::new_with_optional_location(
                ExpressionBody::Logical(LogicalExpression {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                outer_location,
            )
        },
    )(input)
}

fn expression_type_predicate_operator<'s, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<BinaryOperator, E> {
    context(
        "expression_type_predicate_operator",
        alt((value(BinaryOperator::Is, tag("is")),)),
    )(input)
}

fn expression_type_predicate<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    let (input, init) = context("expression_type_predicate_lhs", expression_type_additive)(input)?;

    let Ok((input, (op, t))) = tuple((
        ws(expression_type_predicate_operator::<E>),
        expression_numeric_predicative,
    ))(input) else {
        return Ok((input, init));
    };

    let outer_location = init
        .location
        .and_then(|l| t.location.map(|r| Location::new(l.start, r.end)));

    Ok((
        input,
        Expression::new_with_optional_location(
            ExpressionBody::Binary(BinaryExpression {
                operator: op,
                left: Box::new(init),
                right: Box::new(t),
            }),
            outer_location,
        ),
    ))
}

fn expression_type_additive_operator<'s, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<BinaryOperator, E> {
    context(
        "expression_type_additive_operator",
        alt((value(BinaryOperator::Cast, tag("as")),)),
    )(input)
}

fn expression_type_additive<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    let (input, init) = context("expression_type_additive", expression_numeric_predicative)(input)?;

    fold_many0(
        pair(
            ws(expression_type_additive_operator),
            context(
                "expression_type_additive_rhs",
                expression_numeric_predicative,
            ),
        ),
        move || init.clone(),
        |left, (operator, right)| {
            let outer_location = left
                .location
                .and_then(|l| right.location.map(|r| Location::new(l.start, r.end)));

            Expression::new_with_optional_location(
                ExpressionBody::Binary(BinaryExpression {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                outer_location,
            )
        },
    )(input)
}

fn expression_numeric_predicative_operator<'s, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<BinaryOperator, E> {
    context(
        "expression_numeric_predicative_operator",
        alt((
            value(BinaryOperator::GreaterThanEqual, tag(">=")),
            value(BinaryOperator::LessThanEqual, tag("<=")),
            value(BinaryOperator::LessThan, tag("<")),
            value(BinaryOperator::GreaterThan, tag(">")),
            value(BinaryOperator::StrictEqual, tag("==")),
            value(BinaryOperator::StrictNotEqual, tag("!=")),
            value(BinaryOperator::In, pair(tag("in"), peek(not(alpha1)))),
        )),
    )(input)
}

fn expression_numeric_predicative<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    let (input, init) = context(
        "expression_numeric_predicative_lhs",
        expression_numeric_additive,
    )(input)?;

    fold_many0(
        pair(
            ws(expression_numeric_predicative_operator),
            context(
                "expression_numeric_predicative_rhs",
                expression_numeric_additive,
            ),
        ),
        move || init.clone(),
        |left, (operator, right)| {
            let outer_location = left
                .location
                .and_then(|l| right.location.map(|r| Location::new(l.start, r.end)));

            Expression::new_with_optional_location(
                ExpressionBody::Binary(BinaryExpression {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                outer_location,
            )
        },
    )(input)
}

fn expression_numeric_additive_operator<'s, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<BinaryOperator, E> {
    context(
        "expression_numeric_additive_operator",
        alt((
            value(BinaryOperator::Plus, tag("+")),
            value(BinaryOperator::Minus, tag("-")),
        )),
    )(input)
}

fn expression_numeric_additive<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    let (input, init) = context(
        "expression_numeric_additive_lhs",
        expression_numeric_multiplicative,
    )(input)?;

    fold_many0(
        pair(
            ws(expression_numeric_additive_operator),
            context(
                "expression_numeric_additive_rhs",
                expression_numeric_multiplicative,
            ),
        ),
        move || init.clone(),
        |left, (operator, right)| {
            let outer_location = left
                .location
                .and_then(|l| right.location.map(|r| Location::new(l.start, r.end)));

            Expression::new_with_optional_location(
                ExpressionBody::Binary(BinaryExpression {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                outer_location,
            )
        },
    )(input)
}

fn expression_numeric_multiplicative_operator<'s, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<BinaryOperator, E> {
    context(
        "expression_numeric_multiplicative_operator",
        alt((
            value(BinaryOperator::Times, tag("*")),
            value(BinaryOperator::Over, tag("/")),
            value(BinaryOperator::Mod, tag("%")),
        )),
    )(input)
}

fn expression_numeric_multiplicative<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    let (input, init) = context(
        "expression_numeric_multiplicative_lhs",
        expression_numeric_exponential,
    )(input)?;

    fold_many0(
        pair(
            ws(expression_numeric_multiplicative_operator),
            context(
                "expression_numeric_multiplicative_lhs",
                expression_numeric_exponential,
            ),
        ),
        move || init.clone(),
        |left, (operator, right)| {
            let outer_location = left
                .location
                .and_then(|l| right.location.map(|r| Location::new(l.start, r.end)));

            Expression::new_with_optional_location(
                ExpressionBody::Binary(BinaryExpression {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                outer_location,
            )
        },
    )(input)
}

fn expression_numeric_exponential_operator<'s, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<BinaryOperator, E> {
    context(
        "expression_numeric_exponential_operator",
        alt((value(BinaryOperator::PowerOf, tag("^")),)),
    )(input)
}

fn expression_numeric_exponential<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    let (input, init) = context("expression_numeric_exponential_lhs", expression_path)(input)?;

    fold_many0(
        pair(
            ws(expression_numeric_exponential_operator),
            context("expression_numeric_exponential_rhs", expression_path),
        ),
        move || init.clone(),
        |left, (operator, right)| {
            let outer_location = left
                .location
                .and_then(|l| right.location.map(|r| Location::new(l.start, r.end)));

            Expression::new_with_optional_location(
                ExpressionBody::Binary(BinaryExpression {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                outer_location,
            )
        },
    )(input)
}

enum PathSegment<'a> {
    Application(Expression<'a>, Location),
    Index(Expression<'a>, Location),
    Prop(Identifier<'a>, Location),
}

fn expression_path<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    let (input, init) = context("expression_path_lhs", expression_primary)(input)?;

    fold_many0(
        alt((
            map(
                tuple((
                    position,
                    delimited(
                        ws(tag("[")),
                        context("expression_path_rhs", expression),
                        ws(tag("]")),
                    ),
                    position,
                )),
                |(ls, expr, rs)| {
                    PathSegment::Index(
                        expr,
                        Location::new(ls.location_offset(), rs.location_offset()),
                    )
                },
            ),
            map(
                preceded(
                    ws(tag(".")),
                    context(
                        "expression_member_rhs",
                        tuple((position, identifier, position)),
                    ),
                ),
                |(ls, id, rs)| {
                    PathSegment::Prop(
                        id,
                        Location::new(ls.location_offset(), rs.location_offset()),
                    )
                },
            ),
            map(
                tuple((
                    ws(terminated(position, tag("."))),
                    expression_with_paren,
                    position,
                )),
                |(ls, expr, rs)| {
                    PathSegment::Application(
                        expr,
                        Location::new(ls.location_offset(), rs.location_offset()),
                    )
                },
            ),
        )),
        move || init.clone(),
        |acc, segment| {
            let right_location = match segment {
                PathSegment::Application(_, loc) => loc,
                PathSegment::Index(_, loc) => loc,
                PathSegment::Prop(_, loc) => loc,
            };

            let outer_location = acc
                .location
                .map(|l| Location::new(l.start, right_location.end));

            Expression::new_with_optional_location(
                match segment {
                    PathSegment::Application(param, _) => {
                        ExpressionBody::Application(LambdaApplication {
                            lambda: Box::new(acc),
                            parameter: Box::new(param),
                        })
                    }
                    PathSegment::Index(expr, _) => ExpressionBody::Member(MemberExpression {
                        object: Box::new(acc),
                        property: Box::new(expr),
                    }),
                    PathSegment::Prop(id, loc) => ExpressionBody::Member(MemberExpression {
                        object: Box::new(acc),
                        property: Box::new(Expression::new_with_location(
                            ExpressionBody::Literal(Literal::String(id.name)),
                            loc,
                        )),
                    }),
                },
                outer_location,
            )
        },
    )(input)
}

fn expression_primary<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_primary",
        alt((
            expression_with_paren,
            expression_literal,
            expression_identifier,
            expression_unary,
        )),
    )(input)
}

fn expression_with_paren<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_with_paren",
        delimited(tag("("), expression, tag(")")),
    )(input)
}

fn expression_unary<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression_unary",
        alt((expression_unary_logic, expression_unary_numeric)),
    )(input)
}

fn expression_unary_logic_operator<'s, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<UnaryOperator, E> {
    context(
        "expression_unary_logic_operator",
        alt((value(UnaryOperator::Not, tag("!")),)),
    )(input)
}

fn expression_unary_logic<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    located_expression(map(
        pair(
            ws(expression_unary_logic_operator),
            context("expression_unary_logic_operand", expression_primary),
        ),
        |(operator, argument)| {
            ExpressionBody::Unary(UnaryExpression {
                operator,
                argument: Box::new(argument),
            })
        },
    ))(input)
}

fn expression_unary_numeric_operator<'s, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<UnaryOperator, E> {
    context(
        "expression_unary_numeric_operator",
        alt((
            value(UnaryOperator::Minus, tag("-")),
            value(UnaryOperator::Plus, tag("+")),
        )),
    )(input)
}

fn expression_unary_numeric<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    located_expression(map(
        pair(
            ws(expression_unary_numeric_operator),
            context("expression_unary_numeric_operand", expression_path),
        ),
        |(operator, argument)| {
            ExpressionBody::Unary(UnaryExpression {
                operator,
                argument: Box::new(argument),
            })
        },
    ))(input)
}

pub fn expression<'v, 's, E: ParserError<'s>>(
    input: ParserInput<'s>,
) -> ParserResult<Expression<'v>, E> {
    context(
        "expression",
        alt((
            expression_lambda_match_abstraction,
            expression_lambda_abstraction,
            expression_match,
            expression_condition,
            expression_logic_additive,
        )),
    )(input)
}
