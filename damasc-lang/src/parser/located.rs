use crate::syntax::pattern::PatternBody;
use crate::syntax::pattern::Pattern;
use crate::syntax::expression::ExpressionBody;
use nom::combinator::map;
use nom::sequence::tuple;
use crate::syntax::expression::Expression;
use crate::parser::io::ParserInput;
use crate::parser::io::ParserError;
use crate::syntax::location::Location;


use nom::IResult;
use nom_locate::position;



pub fn located_expression<'s, 'v, F, E: ParserError<'s>>(
    expression_body: F,
) -> impl FnMut(ParserInput<'s>) -> IResult<ParserInput<'s>, Expression<'v>, E>
where
    F: FnMut(ParserInput<'s>) -> IResult<ParserInput<'s>, ExpressionBody<'v>, E>,
{
    map(
        tuple((position, expression_body, position)),
        |(start, body, end)| {
            Expression::new_with_location(
                body,
                Location::new(start.location_offset(), end.location_offset()),
            )
        },
    )
}


pub fn located_pattern<'s, 'v, F, E: ParserError<'s>>(
    pattern_body: F,
) -> impl FnMut(ParserInput<'s>) -> IResult<ParserInput<'s>, Pattern<'v>, E>
where
    F: FnMut(ParserInput<'s>) -> IResult<ParserInput<'s>, PatternBody<'v>, E>,
{
    map(
        tuple((position, pattern_body, position)),
        |(start, body, end)| {
            Pattern::new_with_location(
                body,
                Location::new(start.location_offset(), end.location_offset()),
            )
        },
    )
}