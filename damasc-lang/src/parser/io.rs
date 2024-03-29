use std::fmt::Debug;

use nom::{
    error::{Error, VerboseError},
    IResult,
};
use nom_locate::LocatedSpan;

use super::error::DamascSyntaxError;

pub trait ParserError<'s>:
    nom::error::ContextError<ParserInput<'s>> + nom::error::ParseError<ParserInput<'s>> + Debug
{
}

pub type ParserInput<'s> = LocatedSpan<&'s str>;

pub type ParserResult<'s, T, E = DamascSyntaxError<ParserInput<'s>>> =
    IResult<ParserInput<'s>, T, E>;

impl<'s> ParserError<'s> for VerboseError<ParserInput<'s>> {}
impl<'s> ParserError<'s> for Error<ParserInput<'s>> {}
impl<'s> ParserError<'s> for DamascSyntaxError<ParserInput<'s>> {}
