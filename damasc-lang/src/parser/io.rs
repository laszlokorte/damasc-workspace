use nom::{IResult, error::VerboseError};
use nom_locate::LocatedSpan;

pub type ParserInput<'s> = LocatedSpan<&'s str>;

pub type ParserResult<'s, T> = IResult<ParserInput<'s>, T, VerboseError<ParserInput<'s>>>;

