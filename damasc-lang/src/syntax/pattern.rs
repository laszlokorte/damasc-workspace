use crate::identifier::Identifier;
use crate::literal::Literal;
use crate::syntax::expression::Expression;
use crate::syntax::expression::PropertyKey;
use crate::syntax::location::Location;
use crate::value_type::ValueType;
use core::hash::Hash;
use core::hash::Hasher;

#[derive(Clone, Debug, PartialOrd)]
pub struct Pattern<'s> {
    pub body: PatternBody<'s>,
    pub location: Option<Location>,
}

impl PartialEq for Pattern<'_> {
    fn eq(&self, other: &Pattern<'_>) -> bool {
        self.body.eq(&other.body)
    }
}

impl Eq for Pattern<'_> {}

impl Hash for Pattern<'_> {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.body.hash(hasher)
    }
}

impl Ord for Pattern<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.body.cmp(&other.body)
    }
}

impl<'s> Pattern<'s> {
    pub fn new(body: PatternBody<'s>) -> Pattern<'s> {
        Self {
            body,
            location: None,
        }
    }

    pub fn new_with_location(body: PatternBody<'s>, location: Location) -> Pattern<'s> {
        Self {
            body,
            location: Some(location),
        }
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum PatternBody<'s> {
    Discard,
    Capture(Identifier<'s>, Box<Pattern<'s>>),
    Identifier(Identifier<'s>),
    PinnedExpression(Box<Expression<'s>>),
    TypedDiscard(ValueType),
    TypedIdentifier(Identifier<'s>, ValueType),
    Literal(Literal<'s>),
    Object(ObjectPattern<'s>, Rest<'s>),
    Array(ArrayPattern<'s>, Rest<'s>),
}

impl<'s> Pattern<'s> {
    pub fn deep_clone<'x>(&self) -> Pattern<'x> {
        Pattern {
            location: self.location,
            body: match &self.body {
                PatternBody::Discard => PatternBody::Discard,
                PatternBody::Capture(i, p) => {
                    PatternBody::Capture(i.deep_clone(), Box::new(p.deep_clone()))
                }
                PatternBody::Identifier(i) => PatternBody::Identifier(i.deep_clone()),
                PatternBody::TypedDiscard(t) => PatternBody::TypedDiscard(*t),
                PatternBody::TypedIdentifier(i, t) => PatternBody::TypedIdentifier(i.deep_clone(), *t),
                PatternBody::Literal(l) => PatternBody::Literal(l.deep_clone()),
                PatternBody::Object(pat, rst) => PatternBody::Object(
                    pat.iter().map(|e| e.deep_clone()).collect(),
                    rst.deep_clone(),
                ),
                PatternBody::Array(pat, rst) => PatternBody::Array(
                    pat.iter().map(|e| e.deep_clone()).collect(),
                    rst.deep_clone(),
                ),
                PatternBody::PinnedExpression(e) => {
                    PatternBody::PinnedExpression(Box::new(e.deep_clone()))
                }
            }
        }
    }
}

impl<'a> std::fmt::Display for Pattern<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = match &self.body {
            PatternBody::Discard => write!(f, "_"),
            PatternBody::Literal(l) => write!(f, "{l}"),
            PatternBody::Capture(id, pat) => write!(f, "{id} @ {pat}"),
            PatternBody::TypedDiscard(t) => write!(f, "_ is {t}"),
            PatternBody::Identifier(id) => write!(f, "{id}"),
            PatternBody::PinnedExpression(expr) => write!(f, "^{expr}"),
            PatternBody::TypedIdentifier(id, t) => write!(f, "{id} is {t}"),
            PatternBody::Object(props, rest) => {
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
            PatternBody::Array(items, rest) => {
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
