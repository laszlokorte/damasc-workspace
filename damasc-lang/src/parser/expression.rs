use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{alpha1, space0},
    combinator::{all_consuming, map, not, opt, peek, recognize, value},
    multi::{fold_many0, many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple}, error::context,
};

use crate::{
    identifier::Identifier,
    literal::Literal,
    syntax::expression::{
        ArrayItem, BinaryExpression, BinaryOperator, CallExpression, Expression, ExpressionSet,
        LogicalExpression, LogicalOperator, MemberExpression, ObjectProperty, Property,
        PropertyKey, StringTemplate, StringTemplatePart, UnaryExpression, UnaryOperator,
    },
};

use super::{
    identifier::identifier,
    literal::{literal, literal_string_raw},
    util::ws, io::{ParserResult, ParserInput},
};

pub fn expression_all_consuming<'v>(input: &str) -> Option<Expression<'v>> {
    match all_consuming(expression)(ParserInput::new(input)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

pub fn expression_many1<'v>(input: ParserInput) -> ParserResult<ExpressionSet<'v>> {
    delimited(
        space0,
        map(separated_list1(ws(tag(";")), expression), |expressions| {
            ExpressionSet { expressions }
        }),
        ws(opt(tag(";"))),
    )(input)
}

pub fn expression_many1_all_consuming<'v>(input: &str) -> Option<ExpressionSet<'v>> {
    match all_consuming(expression_many1)(ParserInput::new(input)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

pub fn expression_many0_all_consuming<'v>(input: &str) -> Option<ExpressionSet<'v>> {
    match all_consuming(expression_many0)(ParserInput::new(input)) {
        Ok((_, r)) => Some(r),
        Err(_) => None,
    }
}

pub fn expression_many0<'v>(input: ParserInput) -> ParserResult<ExpressionSet<'v>> {
    delimited(
        space0,
        map(separated_list0(ws(tag(";")), expression), |expressions| {
            ExpressionSet { expressions }
        }),
        ws(opt(tag(";"))),
    )(input)
}

fn expression_array_item<'v>(input: ParserInput) -> ParserResult<ArrayItem<'v>> {
    context("expression_array_item", alt((
        map(preceded(ws(tag("...")), expression), ArrayItem::Spread),
        map(expression, ArrayItem::Single),
    )))(input)
}

fn expression_call<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    context("expression_call", map(
        pair(
            identifier,
            delimited(ws(tag("(")), expression, ws(tag(")"))),
        ),
        |(function, arg)| {
            Expression::Call(CallExpression {
                function,
                argument: Box::new(arg),
            })
        },
    ))(input)
}

fn expression_array<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    context("expression_array", delimited(
        ws(tag("[")),
        terminated(
            map(
                separated_list0(ws(tag(",")), expression_array_item),
                Expression::Array,
            ),
            opt(ws(tag(","))),
        ),
        ws(tag("]")),
    ))(input)
}

fn expression_object_property<'v>(input: ParserInput) -> ParserResult<ObjectProperty<'v>> {
    context("expression_object_property", alt((
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
    )))(input)
}

fn expression_object<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    context("expression_object", delimited(
        ws(tag("{")),
        terminated(
            map(
                separated_list0(ws(ws(tag(","))), expression_object_property),
                Expression::Object,
            ),
            opt(ws(tag(","))),
        ),
        ws(tag("}")),
    ))(input)
}

fn expression_literal<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    context("expression_literal", alt((
        expression_object,
        expression_array,
        expression_string_template,
        expression_call,
        expression_atom,
    )))(input)
}

fn expression_atom<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    context("expression_atom", map(literal, Expression::Literal))(input)
}

fn expression_identifier<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    context("expression_identifier", map(identifier, Expression::Identifier))(input)
}

fn string_template_part<'v>(input: ParserInput) -> ParserResult<StringTemplatePart<'v>> {
    context("expression_string_template_part", map(
        tuple((
            recognize(take_until("${")),
            delimited(tag("${"), expression, tag("}")),
        )),
        |(fixed_start, dynamic_end)| StringTemplatePart {
            fixed_start: Cow::Owned(fixed_start.fragment().to_owned().into()),
            dynamic_end: Box::new(dynamic_end),
        },
    ))(input)
}

fn expression_string_template<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    context("expression_string_template", map(
        delimited(
            tag("`"),
            tuple((many0(string_template_part), recognize(many0(is_not("`"))))),
            tag("`"),
        ),
        |(parts, s)| {
            Expression::Template(StringTemplate {
                parts,
                suffix: Cow::Owned(s.to_string()),
            })
        },
    ))(input)
}

fn expression_logic_additive_operator(input: ParserInput) -> ParserResult<LogicalOperator> {
    context("expression_logic_operator", alt((value(LogicalOperator::Or, tag("||")),)))(input)
}

fn expression_logic_additive<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    let (input, init) = expression_logic_multiplicative(input)?;

    fold_many0(
        pair(
            ws(expression_logic_additive_operator),
            expression_logic_multiplicative,
        ),
        move || init.clone(),
        |left, (operator, right)| {
            Expression::Logical(LogicalExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            })
        },
    )(input)
}


fn expression_logic_multiplicative_operator(input: ParserInput) -> ParserResult<LogicalOperator> {
    context("expression_logic_operator", 
    alt((value(LogicalOperator::And, tag("&&")),)))(input)
}


fn expression_logic_multiplicative<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    let (input, init) = expression_type_predicate(input)?;

    fold_many0(
        pair(
            ws(expression_logic_multiplicative_operator),
            expression_type_predicate,
        ),
        move || init.clone(),
        |left, (operator, right)| {
            Expression::Logical(LogicalExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            })
        },
    )(input)
}

fn expression_type_predicate_operator(input: ParserInput) -> ParserResult<BinaryOperator> {
    context("expression_type_predicate_operator", alt((
        value(BinaryOperator::Is, tag("is")),
    )))(input)
}

fn expression_type_predicate<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    let (input, init) = expression_type_additive(input)?;

    let Ok((input, (op, t))) = tuple((ws(expression_type_predicate_operator), expression_numeric_predicative))(input) else {
        return Ok((input, init));
    };

    Ok((
        input,
        Expression::Binary(BinaryExpression {
            operator: op,
            left: Box::new(init),
            right: Box::new(t),
        }),
    ))
}

fn expression_type_additive_operator(input: ParserInput) -> ParserResult<BinaryOperator> {
    context("expression_type_additive_operator", alt((
        value(BinaryOperator::Cast, tag("as")),
    )))(input)
}

fn expression_type_additive<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    let (input, init) = expression_numeric_predicative(input)?;

    fold_many0(
        pair(
            ws(expression_type_additive_operator),
            expression_numeric_predicative,
        ),
        move || init.clone(),
        |left, (operator, right)| {
            Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            })
        },
    )(input)
}

fn expression_numeric_predicative_operator(input: ParserInput) -> ParserResult<BinaryOperator> {
    context("expression_numeric_predicative_operator", alt((
        value(BinaryOperator::GreaterThanEqual, tag(">=")),
        value(BinaryOperator::LessThanEqual, tag("<=")),
        value(BinaryOperator::LessThan, tag("<")),
        value(BinaryOperator::GreaterThan, tag(">")),
        value(BinaryOperator::StrictEqual, tag("==")),
        value(BinaryOperator::StrictNotEqual, tag("!=")),
        value(BinaryOperator::In, pair(tag("in"), peek(not(alpha1)))),
    )))(input)
}

fn expression_numeric_predicative<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    let (input, init) = expression_numeric_additive(input)?;

    fold_many0(
        pair(
            ws(expression_numeric_predicative_operator),
            expression_numeric_additive,
        ),
        move || init.clone(),
        |left, (operator, right)| {
            Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            })
        },
    )(input)
}


fn expression_numeric_additive_operator(input: ParserInput) -> ParserResult<BinaryOperator> {
    context("expression_numeric_additive_operator", alt((
        value(BinaryOperator::Plus, tag("+")),
        value(BinaryOperator::Minus, tag("-")),
    )))(input)
}

fn expression_numeric_additive<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    let (input, init) = expression_numeric_multiplicative(input)?;

    fold_many0(
        pair(
            ws(expression_numeric_additive_operator),
            expression_numeric_multiplicative,
        ),
        move || init.clone(),
        |left, (operator, right)| {
            Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            })
        },
    )(input)
}


fn expression_numeric_multiplicative_operator(input: ParserInput) -> ParserResult<BinaryOperator> {
    context("expression_numeric_multiplicative_operator", alt((
        value(BinaryOperator::Times, tag("*")),
        value(BinaryOperator::Over, tag("/")),
        value(BinaryOperator::Mod, tag("%")),
    )))(input)
}

fn expression_numeric_multiplicative<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    let (input, init) = expression_numeric_exponential(input)?;

    fold_many0(
        pair(
            ws(expression_numeric_multiplicative_operator),
            expression_numeric_exponential,
        ),
        move || init.clone(),
        |left, (operator, right)| {
            Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            })
        },
    )(input)
}


fn expression_numeric_exponential_operator(input: ParserInput) -> ParserResult<BinaryOperator> {
    context("expression_numeric_exponential_operator", 
    alt((value(BinaryOperator::PowerOf, tag("^")),)))(input)
}

fn expression_numeric_exponential<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    let (input, init) = expression_indexed(input)?;

    fold_many0(
        pair(
            ws(expression_numeric_exponential_operator),
            expression_indexed,
        ),
        move || init.clone(),
        |left, (operator, right)| {
            Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            })
        },
    )(input)
}

fn expression_indexed<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    let (input, init) = expression_member(input)?;

    fold_many0(
        delimited(ws(tag("[")), expression, ws(tag("]"))),
        move || init.clone(),
        |acc, ident| {
            Expression::Member(MemberExpression {
                object: Box::new(acc),
                property: Box::new(ident),
            })
        },
    )(input)
}

fn expression_member<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    let (input, init) = expression_primary(input)?;

    fold_many0(
        alt((preceded(ws(tag(".")), identifier),)),
        move || init.clone(),
        |acc, ident| {
            Expression::Member(MemberExpression {
                object: Box::new(acc),
                property: Box::new(Expression::Literal(Literal::String(ident.name))),
            })
        },
    )(input)
}

fn expression_primary<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    alt((
        expression_with_paren,
        expression_literal,
        expression_identifier,
        expression_unary,
    ))(input)
}

fn expression_with_paren<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    context("expression_with_paren", delimited(tag("("), expression, tag(")")))(input)
}

fn expression_unary<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    context("expression_unary", alt((expression_unary_logic, expression_unary_numeric)))(input)
}

fn expression_unary_logic_operator(input: ParserInput) -> ParserResult<UnaryOperator> {
    context("expression_unary_logic_operator", 
    alt((value(UnaryOperator::Not, tag("!")),)))(input)
}

fn expression_unary_logic<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    map(
        pair(
            ws(expression_unary_logic_operator),
            expression_primary,
        ),
        |(operator, argument)| {
            Expression::Unary(UnaryExpression {
                operator,
                argument: Box::new(argument),
            })
        },
    )(input)
}


fn expression_unary_numeric_operator(input: ParserInput) -> ParserResult<UnaryOperator> {
    context("expression_unary_numeric_operator", 
    alt((
        value(UnaryOperator::Minus, tag("-")),
        value(UnaryOperator::Plus, tag("+")),
    )))(input)
}

fn expression_unary_numeric<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    map(
        pair(
            ws(expression_unary_numeric_operator ),
            alt((expression_indexed,)),
        ),
        |(operator, argument)| {
            Expression::Unary(UnaryExpression {
                operator,
                argument: Box::new(argument),
            })
        },
    )(input)
}

pub fn expression<'v>(input: ParserInput) -> ParserResult<Expression<'v>> {
    context("expression", expression_logic_additive)(input)
}
