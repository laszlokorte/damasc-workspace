use std::borrow::Cow;

use crate::value_type::ValueType;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal<'s> {
    Null,
    String(Cow<'s, str>),
    Number(Cow<'s, str>),
    Boolean(bool),
    Type(ValueType),
}
impl Literal<'_> {
    pub(crate) fn deep_clone<'x>(&self) -> Literal<'x> {
        match self {
            Literal::Null => Literal::Null,
            Literal::String(s) => Literal::String(Cow::Owned(s.to_string())),
            Literal::Number(s) => Literal::Number(Cow::Owned(s.to_string())),
            Literal::Boolean(b) => Literal::Boolean(*b),
            Literal::Type(t) => Literal::Type(*t),
        }
    }
}

impl<'a> std::fmt::Display for Literal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Null => write!(f, "null"),
            Literal::String(s) => write!(f, "\"{s}\""),
            Literal::Number(n) => write!(f, "{n}"),
            Literal::Boolean(b) => write!(f, "{b}"),
            Literal::Type(t) => write!(f, "{t}"),
        }
    }
}
