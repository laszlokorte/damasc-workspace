use crate::identifier::Identifier;
use crate::literal::Literal;
use crate::syntax::expression::PropertyKey;
use crate::value::ValueType;

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum Pattern<'s> {
    Discard,
    Capture(Identifier<'s>, Box<Pattern<'s>>),
    Identifier(Identifier<'s>),
    TypedDiscard(ValueType),
    TypedIdentifier(Identifier<'s>, ValueType),
    Literal(Literal<'s>),
    Object(ObjectPattern<'s>, Rest<'s>),
    Array(ArrayPattern<'s>, Rest<'s>),
}
impl <'s> Pattern<'s> {
    pub(crate) fn deep_clone<'x>(&self) -> Pattern<'x> {
        match self {
            Pattern::Discard => Pattern::Discard,
            Pattern::Capture(i, p) => Pattern::Capture(i.deep_clone(), Box::new(p.deep_clone())),
            Pattern::Identifier(i) => Pattern::Identifier(i.deep_clone()),
            Pattern::TypedDiscard(t) => Pattern::TypedDiscard(*t),
            Pattern::TypedIdentifier(i, t) => Pattern::TypedIdentifier(i.deep_clone(), *t),
            Pattern::Literal(l) => Pattern::Literal(l.deep_clone()),
            Pattern::Object(pat, rst) => Pattern::Object(pat.iter().map(|e| e.deep_clone()).collect(), rst.deep_clone()),
            Pattern::Array(pat, rst) => Pattern::Array(pat.iter().map(|e| e.deep_clone()).collect(), rst.deep_clone()),
        }
    }
}

impl<'a> std::fmt::Display for Pattern<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = match self {
            Pattern::Discard => write!(f, "_"),
            Pattern::Literal(l) => write!(f, "{l}"),
            Pattern::Capture(id, pat) => write!(f, "{pat} @ {id}"),
            Pattern::TypedDiscard(t) => write!(f, "_ is {t}"),
            Pattern::Identifier(id) => write!(f, "{id}"),
            Pattern::TypedIdentifier(id, t) => write!(f, "{id} is {t}"),
            Pattern::Object(props, rest) => {
                let _ = write!(f, "{{");

                for prop in props {
                    let _ = match prop {
                        ObjectPropertyPattern::Single(p) => write!(f, "{p}"),
                        ObjectPropertyPattern::Match(PropertyPattern { key, value }) => {
                            let _ = match key {
                                PropertyKey::Identifier(id) => {
                                    write!(f, "{id}")
                                }
                                PropertyKey::Expression(e) => {
                                    write!(f, "{e}")
                                }
                            };

                            write!(f, ": {value}")
                        }
                    };
                    let _ = write!(f, ",");
                }

                match rest {
                    Rest::Exact => {}
                    Rest::Discard => {
                        let _ = write!(f, "...");
                    }
                    Rest::Collect(p) => {
                        let _ = write!(f, "...{p}");
                    }
                };

                write!(f, "}}")
            }
            Pattern::Array(items, rest) => {
                let _ = write!(f, "[");
                for ArrayPatternItem::Pattern(item) in items {
                    let _ = write!(f, "{item},");
                }

                match rest {
                    Rest::Exact => {}
                    Rest::Discard => {
                        let _ = write!(f, "...");
                    }
                    Rest::Collect(p) => {
                        let _ = write!(f, "...{p}");
                    }
                };
                write!(f, "]")
            }
        };
        write!(f, "")
    }
}

#[derive(Clone, Debug)]
pub struct PatternSet<'s> {
    pub patterns: Vec<Pattern<'s>>,
}

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum Rest<'s> {
    Exact,
    Discard,
    Collect(Box<Pattern<'s>>),
}
impl Rest<'_> {
    fn deep_clone<'x>(&self) -> Rest<'x> {
        match self {
            Rest::Exact => Rest::Exact,
            Rest::Discard => Rest::Discard,
            Rest::Collect(p) => Rest::Collect(Box::new(p.deep_clone())),
        }
    }
}

pub type ObjectPattern<'a> = Vec<ObjectPropertyPattern<'a>>;
pub type ArrayPattern<'a> = Vec<ArrayPatternItem<'a>>;

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum ArrayPatternItem<'a> {
    Pattern(Pattern<'a>),
    //Expression(Expression<'a>),
}
impl ArrayPatternItem<'_> {
    fn deep_clone<'x>(&self) -> ArrayPatternItem<'x> {
        match self {
            ArrayPatternItem::Pattern(p) => ArrayPatternItem::Pattern(p.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum ObjectPropertyPattern<'a> {
    Single(Identifier<'a>),
    Match(PropertyPattern<'a>),
}
impl ObjectPropertyPattern<'_> {
    fn deep_clone<'x>(&self) -> ObjectPropertyPattern<'x> {
        match self {
            ObjectPropertyPattern::Single(s) => ObjectPropertyPattern::Single(s.deep_clone()),
            ObjectPropertyPattern::Match(m) => ObjectPropertyPattern::Match(m.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub struct PropertyPattern<'a> {
    pub key: PropertyKey<'a>,
    pub value: Pattern<'a>,
}
impl PropertyPattern<'_> {
    fn deep_clone<'x>(&self) -> PropertyPattern<'x> {
        PropertyPattern { 
            key: self.key.deep_clone(), 
            value: self.value.deep_clone(),
        }
    }
}
