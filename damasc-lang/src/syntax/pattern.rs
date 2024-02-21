use crate::syntax::level::EmptyLevel;
use crate::syntax::level::SyntaxLevel;
use crate::syntax::expression::Expression;
use crate::identifier::Identifier;
use crate::literal::Literal;
use crate::syntax::expression::PropertyKey;
use crate::value_type::ValueType;

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum Pattern<'s, Level: SyntaxLevel = EmptyLevel> {
    Discard,
    Capture(Identifier<'s>, Box<Pattern<'s, Level>>),
    Identifier(Identifier<'s>),
    PinnedExpression(Box<Expression<'s, Level>>),
    TypedDiscard(ValueType),
    TypedIdentifier(Identifier<'s, >, ValueType),
    Literal(Literal<'s>),
    Object(ObjectPattern<'s, Level>, Rest<'s, Level>),
    Array(ArrayPattern<'s, Level>, Rest<'s, Level>),
}
impl<'s, Level: SyntaxLevel> Pattern<'s, Level> {
    pub(crate) fn deep_clone<'x>(&self) -> Pattern<'x, Level> {
        match self {
            Pattern::Discard => Pattern::Discard,
            Pattern::Capture(i, p) => Pattern::Capture(i.deep_clone(), Box::new(p.deep_clone())),
            Pattern::Identifier(i) => Pattern::Identifier(i.deep_clone()),
            Pattern::TypedDiscard(t) => Pattern::TypedDiscard(*t),
            Pattern::TypedIdentifier(i, t) => Pattern::TypedIdentifier(i.deep_clone(), *t),
            Pattern::Literal(l) => Pattern::Literal(l.deep_clone()),
            Pattern::Object(pat, rst) => Pattern::Object(
                pat.iter().map(|e| e.deep_clone()).collect(),
                rst.deep_clone(),
            ),
            Pattern::Array(pat, rst) => Pattern::Array(
                pat.iter().map(|e| e.deep_clone()).collect(),
                rst.deep_clone(),
            ),
            Pattern::PinnedExpression(e) => Pattern::PinnedExpression(Box::new(e.deep_clone())),
        }
    }
}

impl<'a, Level: SyntaxLevel> std::fmt::Display for Pattern<'a, Level> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = match self {
            Pattern::Discard => write!(f, "_"),
            Pattern::Literal(l) => write!(f, "{l}"),
            Pattern::Capture(id, pat) => write!(f, "{id} @ {pat}"),
            Pattern::TypedDiscard(t) => write!(f, "_ is {t}"),
            Pattern::Identifier(id) => write!(f, "{id}"),
            Pattern::PinnedExpression(expr) => write!(f, "^{expr}"),
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
pub struct PatternSet<'s, Level: SyntaxLevel> {
    pub patterns: Vec<Pattern<'s, Level>>,
}

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum Rest<'s, Level: SyntaxLevel> {
    Exact,
    Discard,
    Collect(Box<Pattern<'s, Level>>),
}
impl<Level: SyntaxLevel> Rest<'_, Level> {
    fn deep_clone<'x>(&self) -> Rest<'x, Level> {
        match self {
            Rest::<Level>::Exact => Rest::Exact,
            Rest::<Level>::Discard => Rest::Discard,
            Rest::<Level>::Collect(p) => Rest::Collect(Box::new(p.deep_clone())),
        }
    }
}

pub type ObjectPattern<'a, Level: SyntaxLevel> = Vec<ObjectPropertyPattern<'a, Level>>;
pub type ArrayPattern<'a, Level: SyntaxLevel> = Vec<ArrayPatternItem<'a, Level>>;

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum ArrayPatternItem<'a, Level: SyntaxLevel> {
    Pattern(Pattern<'a, Level>),
    //Expression(Expression<'a>),
}
impl<Level: SyntaxLevel> ArrayPatternItem<'_, Level> {
    fn deep_clone<'x>(&self) -> ArrayPatternItem<'x, Level> {
        match self {
            ArrayPatternItem::Pattern(p) => ArrayPatternItem::<Level>::Pattern(p.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum ObjectPropertyPattern<'a, Level: SyntaxLevel> {
    Single(Identifier<'a>),
    Match(PropertyPattern<'a, Level>),
}
impl<Level: SyntaxLevel> ObjectPropertyPattern<'_, Level> {
    fn deep_clone<'x>(&self) -> ObjectPropertyPattern<'x, Level> {
        match self {
            ObjectPropertyPattern::Single(s) => ObjectPropertyPattern::Single(s.deep_clone()),
            ObjectPropertyPattern::Match(m) => ObjectPropertyPattern::Match(m.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub struct PropertyPattern<'a, Level: SyntaxLevel> {
    pub key: PropertyKey<'a, Level>,
    pub value: Pattern<'a, Level>,
}
impl<Level: SyntaxLevel> PropertyPattern<'_, Level> {
    fn deep_clone<'x>(&self) -> PropertyPattern<'x, Level> {
        PropertyPattern::<Level> {
            key: self.key.deep_clone(),
            value: self.value.deep_clone(),
        }
    }
}
