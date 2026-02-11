use crate::parser::io::ParserError;
use crate::parser::io::ParserInput;
use crate::syntax::expression::Expression;
use crate::syntax::expression::ExpressionBody;
use crate::syntax::location::Location;
use crate::syntax::pattern::Pattern;
use crate::syntax::pattern::PatternBody;
use nom::combinator::map;
use nom::Parser;

use nom_locate::position;

pub fn located_expression<'s, 'v, F, E: ParserError<'s>>(
    expression_body: F,
) -> impl Parser<ParserInput<'s>, Output = Expression<'v>, Error = E>
where
    F: Parser<ParserInput<'s>, Output = ExpressionBody<'v>, Error = E>,
{
    map(
        (position, expression_body, position),
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
) -> impl Parser<ParserInput<'s>, Output = Pattern<'v>, Error = E>
where
    F: Parser<ParserInput<'s>, Output = PatternBody<'v>, Error = E>,
{
    map((position, pattern_body, position), |(start, body, end)| {
        Pattern::new_with_location(
            body,
            Location::new(start.location_offset(), end.location_offset()),
        )
    })
}
