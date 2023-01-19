use std::borrow::Cow;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{space0, alpha1},
    combinator::{all_consuming, map, opt, recognize, value, not, peek},
    multi::{fold_many0, many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
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
    util::ws,
};

pub fn single_expression<'v>(input: &str) -> Option<Expression<'v>> {
    match all_consuming(expression)(input) {
        Ok((_,r)) => Some(r),
        Err(_) => None,
    }
}

pub fn multi_expressions<'v>(input: &str) -> IResult<&str, ExpressionSet<'v>> {
    delimited(
        space0,
        map(separated_list1(ws(tag(";")), expression), |expressions| {
            ExpressionSet { expressions }
        }),
        ws(opt(tag(";"))),
    )(input)
}

fn array_item_expression<'v>(input: &str) -> IResult<&str, ArrayItem<'v>> {
    alt((
        map(preceded(ws(tag("...")), expression), ArrayItem::Spread),
        map(expression, ArrayItem::Single),
    ))(input)
}

fn expression_call<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    map(
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
    )(input)
}

fn expression_array<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    delimited(
        ws(tag("[")),
        terminated(
            map(
                separated_list0(ws(tag(",")), array_item_expression),
                Expression::Array,
            ),
            opt(ws(tag(","))),
        ),
        ws(tag("]")),
    )(input)
}

fn object_prop_expression<'v>(input: &str) -> IResult<&str, ObjectProperty<'v>> {
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
    ))(input)
}

fn expression_object<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    delimited(
        ws(tag("{")),
        terminated(
            map(
                separated_list0(ws(ws(tag(","))), object_prop_expression),
                Expression::Object,
            ),
            opt(ws(tag(","))),
        ),
        ws(tag("}")),
    )(input)
}

fn expression_literal<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    alt((
        expression_object,
        expression_array,
        expression_string_template,
        expression_call,
        expression_atom,
    ))(input)
}

fn expression_atom<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    map(literal, Expression::Literal)(input)
}

fn expression_identifier<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    map(identifier, Expression::Identifier)(input)
}

fn string_template_part<'v>(input: &str) -> IResult<&str, StringTemplatePart<'v>> {
    map(
        tuple((
            recognize(take_until("${")),
            delimited(tag("${"), expression, tag("}")),
        )),
        |(fixed_start, dynamic_end)| StringTemplatePart {
            fixed_start: Cow::Owned(fixed_start.into()),
            dynamic_end: Box::new(dynamic_end),
        },
    )(input)
}

fn expression_string_template<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    map(
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
    )(input)
}

fn expression_logic_additive<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    let (input, init) = expression_logic_multiplicative(input)?;

    fold_many0(
        pair(
            ws(alt((value(LogicalOperator::Or, tag("||")),))),
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

fn expression_logic_multiplicative<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    let (input, init) = expression_type_predicate(input)?;

    fold_many0(
        pair(
            ws(alt((value(LogicalOperator::And, tag("&&")),))),
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

fn expression_type_predicate<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    let (input, init) = expression_type_additive(input)?;

    let Ok((input, (op, t))) = tuple((ws(alt((
        value(BinaryOperator::Is, tag("is")),
    ))), expression_numeric_predicative))(input) else {
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

fn expression_type_additive<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    let (input, init) = expression_numeric_predicative(input)?;

    fold_many0(
        pair(
            ws(alt((value(BinaryOperator::Cast, tag("as")),))),
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

fn expression_numeric_predicative<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    let (input, init) = expression_numeric_additive(input)?;

    fold_many0(
        pair(
            ws(alt((
                value(BinaryOperator::GreaterThanEqual, tag(">=")),
                value(BinaryOperator::LessThanEqual, tag("<=")),
                value(BinaryOperator::LessThan, tag("<")),
                value(BinaryOperator::GreaterThan, tag(">")),
                value(BinaryOperator::StrictEqual, tag("==")),
                value(BinaryOperator::StrictNotEqual, tag("!=")),
                value(BinaryOperator::In, pair(tag("in"), peek(not(alpha1)))),
            ))),
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

fn expression_numeric_additive<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    let (input, init) = expression_numeric_multiplicative(input)?;

    fold_many0(
        pair(
            ws(alt((
                value(BinaryOperator::Plus, tag("+")),
                value(BinaryOperator::Minus, tag("-")),
            ))),
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

fn expression_numeric_multiplicative<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    let (input, init) = expression_numeric_exponential(input)?;

    fold_many0(
        pair(
            ws(alt((
                value(BinaryOperator::Times, tag("*")),
                value(BinaryOperator::Over, tag("/")),
                value(BinaryOperator::Mod, tag("%")),
            ))),
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

fn expression_numeric_exponential<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    let (input, init) = expression_indexed(input)?;

    fold_many0(
        pair(
            ws(alt((value(BinaryOperator::PowerOf, tag("^")),))),
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

fn expression_indexed<'v>(input: &str) -> IResult<&str, Expression<'v>> {
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

fn expression_member<'v>(input: &str) -> IResult<&str, Expression<'v>> {
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

fn expression_primary<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    alt((
        expression_with_paren,
        expression_literal,
        expression_identifier,
        expression_unary,
    ))(input)
}

fn expression_with_paren<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    delimited(tag("("), expression, tag(")"))(input)
}

fn expression_unary<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    alt((expression_unary_logic, expression_unary_numeric))(input)
}

fn expression_unary_logic<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    map(
        pair(
            ws(alt((value(UnaryOperator::Not, tag("!")),))),
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

fn expression_unary_numeric<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    map(
        pair(
            ws(alt((
                value(UnaryOperator::Minus, tag("-")),
                value(UnaryOperator::Plus, tag("+")),
            ))),
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

pub fn expression<'v>(input: &str) -> IResult<&str, Expression<'v>> {
    alt((expression_logic_additive,))(input)
}
