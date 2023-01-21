use nom::{character::complete::multispace0, error::ParseError, sequence::delimited, IResult};

use super::io::ParserInput;

pub fn ws<'a, F, O, E: ParseError<ParserInput<'a>>>(
    inner: F,
) -> impl FnMut(ParserInput<'a>) -> IResult<ParserInput<'a>, O, E>
where
    F: FnMut(ParserInput<'a>) -> IResult<ParserInput<'a>, O, E>,
{
    delimited(multispace0, inner, multispace0)
}
