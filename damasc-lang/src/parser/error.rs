use nom::error::{ErrorKind, ParseError};

#[derive(Debug)]
pub struct DamascSyntaxError<I> {
    pub kind: SyntaxErrorKind<I>,
}

impl<I> nom::error::ContextError<I> for DamascSyntaxError<I> {
    fn add_context(_input: I, _ctx: &'static str, other: Self) -> Self {
        other
    }
}

#[derive(Debug)]
pub enum SyntaxErrorKind<I> {
    Nom(I, ErrorKind),
    // your error types as the rest of the variants
}

impl<I> ParseError<I> for DamascSyntaxError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            kind: SyntaxErrorKind::Nom(input, kind),
        }
    }

    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> ToString for DamascSyntaxError<I> {
    fn to_string(&self) -> String {
        match self.kind {
            SyntaxErrorKind::Nom(_, e) => e.description().to_string(),
        }
    }
}
