use nom::{character::complete::multispace0, error::ParseError, sequence::delimited, Parser};

use super::io::ParserInput;

pub fn ws<'a, F, O, E: ParseError<ParserInput<'a>>>(
    inner: F,
) -> impl Parser<ParserInput<'a>, Output = O, Error = E>
where
    F: Parser<ParserInput<'a>, Output = O, Error = E>,
{
    delimited(multispace0, inner, multispace0)
}
