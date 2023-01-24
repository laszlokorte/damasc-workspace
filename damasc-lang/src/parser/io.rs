use nom::{IResult, error::{VerboseError, Error, ErrorKind, ParseError}};
use nom_locate::LocatedSpan;


pub trait ParserError<'s> : nom::error::ContextError<ParserInput<'s>> + nom::error::ParseError<ParserInput<'s>> {}

pub type ParserInput<'s> = LocatedSpan<&'s str>;

pub type ParserResult<'s, T, E=VerboseError<ParserInput<'s>>> = IResult<ParserInput<'s>, T, E>;

impl<'s> ParserError<'s> for VerboseError<ParserInput<'s>> {}
impl<'s> ParserError<'s> for Error<ParserInput<'s>> {}
impl<'s> ParserError<'s> for SyntaxError<ParserInput<'s>> {}

impl<'s> nom::error::ContextError<ParserInput<'s>> for SyntaxError<ParserInput<'s>> {
    fn add_context(_input: ParserInput<'s>, _ctx: &'static str, other: Self) -> Self {
        other
    }
}

impl<I> ToString for SyntaxError<I> {
    fn to_string(&self) -> String {
        match self.kind {
            SyntaxErrorKind::Nom(_, e) => e.description().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct SyntaxError<I> {
    pub kind: SyntaxErrorKind<I>,
}


#[derive(Debug)]
pub enum SyntaxErrorKind<I> {
     Nom(I, ErrorKind),
     // your error types as the rest of the variants
}

impl<I> ParseError<I> for SyntaxError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            kind: SyntaxErrorKind::Nom(input, kind),
        }
    }
    
    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other
    }
}