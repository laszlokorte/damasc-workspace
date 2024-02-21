use crate::syntax::expression::AnnotatedIdentifier;
use crate::syntax::expression::AnnotatedLiteral;
use crate::syntax::expression::BoxedExpression;
use crate::syntax::expression::PropertyKey;
use crate::value_type::ValueType;

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub struct AnnotatedPattern<'s, Annotation> {
    pub body: Pattern<'s, Annotation>,
    pub annotation: Annotation,
}

impl<'a, Annotation> std::fmt::Display for AnnotatedPattern<'a, Annotation> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body)
    }
}

pub type BoxedPattern<'s, Annotation> = Box<AnnotatedPattern<'s, Annotation>>;

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum Pattern<'s, Annotation> {
    Discard,
    Capture(AnnotatedIdentifier<'s, Annotation>, BoxedPattern<'s, Annotation>),
    Identifier(AnnotatedIdentifier<'s, Annotation>),
    PinnedExpression(BoxedExpression<'s, Annotation>),
    TypedDiscard(ValueType),
    TypedIdentifier(AnnotatedIdentifier<'s, Annotation>, ValueType),
    Literal(AnnotatedLiteral<'s, Annotation>),
    Object(ObjectPattern<'s, Annotation>, Rest<'s, Annotation>),
    Array(ArrayPattern<'s, Annotation>, Rest<'s, Annotation>),
}
impl<'s, Annotation:Clone> Pattern<'s, Annotation> {
    pub(crate) fn deep_clone<'x>(&self) -> Pattern<'x, Annotation> {
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


impl<'s, Annotation : Clone> AnnotatedPattern<'s, Annotation> {
    pub(crate) fn deep_clone<'x>(&self) -> AnnotatedPattern<'x, Annotation> {
        AnnotatedPattern {
            body: self.body.deep_clone(),
            annotation: self.annotation.clone(),
        }
    }
}

impl<'a, Annotation> std::fmt::Display for Pattern<'a, Annotation> {
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
pub struct PatternSet<'s, Annotation> {
    pub patterns: Vec<Pattern<'s, Annotation>>,
}

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum Rest<'s, Annotation> {
    Exact,
    Discard,
    Collect(BoxedPattern<'s, Annotation>),
}
impl<Annotation: Clone> Rest<'_, Annotation> {
    fn deep_clone<'x>(&self) -> Rest<'x, Annotation> {
        match self {
            Rest::Exact => Rest::Exact,
            Rest::Discard => Rest::Discard,
            Rest::Collect(p) => Rest::Collect(Box::new(p.deep_clone())),
        }
    }
}

pub type ObjectPattern<'a, Annotation> = Vec<ObjectPropertyPattern<'a, Annotation>>;
pub type ArrayPattern<'a, Annotation> = Vec<ArrayPatternItem<'a, Annotation>>;

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum ArrayPatternItem<'a, Annotation> {
    Pattern(AnnotatedPattern<'a, Annotation>),
    //Expression(Expression<'a>),
}
impl<Annotation: Clone> ArrayPatternItem<'_, Annotation> {
    fn deep_clone<'x>(&self) -> ArrayPatternItem<'x, Annotation> {
        match self {
            ArrayPatternItem::Pattern(p) => ArrayPatternItem::Pattern(p.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub enum ObjectPropertyPattern<'a, Annotation> {
    Single(AnnotatedIdentifier<'a, Annotation>),
    Match(PropertyPattern<'a, Annotation>),
}
impl<Annotation:Clone> ObjectPropertyPattern<'_, Annotation> {
    fn deep_clone<'x>(&self) -> ObjectPropertyPattern<'x, Annotation> {
        match self {
            ObjectPropertyPattern::Single(s) => ObjectPropertyPattern::Single(s.deep_clone()),
            ObjectPropertyPattern::Match(m) => ObjectPropertyPattern::Match(m.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, Eq, PartialEq, Hash)]
pub struct PropertyPattern<'a, Annotation> {
    pub key: PropertyKey<'a, Annotation>,
    pub value: AnnotatedPattern<'a, Annotation>,
}

impl<Annotation:Clone> PropertyPattern<'_, Annotation> {
    fn deep_clone<'x>(&self) -> PropertyPattern<'x, Annotation> {
        PropertyPattern {
            key: self.key.deep_clone(),
            value: self.value.deep_clone(),
        }
    }
}
