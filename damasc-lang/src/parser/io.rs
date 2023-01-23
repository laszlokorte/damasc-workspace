use nom::{IResult, error::{VerboseError, Error}};
use nom_locate::LocatedSpan;


pub trait ParserError<'s> : nom::error::ContextError<ParserInput<'s>> + nom::error::ParseError<ParserInput<'s>> {}

pub type ParserInput<'s> = LocatedSpan<&'s str>;

pub type ParserResult<'s, T, E=VerboseError<ParserInput<'s>>> = IResult<ParserInput<'s>, T, E>;

impl<'s> ParserError<'s> for VerboseError<ParserInput<'s>> {}
impl<'s> ParserError<'s> for Error<ParserInput<'s>> {}
