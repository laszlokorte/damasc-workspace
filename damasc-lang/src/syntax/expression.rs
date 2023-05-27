use std::borrow::Cow;

use crate::identifier::Identifier;
use crate::literal::Literal;

use super::pattern::Pattern;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression<'s> {
    Array(ArrayExpression<'s>),
    Binary(BinaryExpression<'s>),
    Identifier(Identifier<'s>),
    Literal(Literal<'s>),
    Logical(LogicalExpression<'s>),
    Member(MemberExpression<'s>),
    Object(ObjectExpression<'s>),
    Unary(UnaryExpression<'s>),
    Call(CallExpression<'s>),
    Template(StringTemplate<'s>),
    Abstraction(LambdaAbstraction<'s>),
    Application(LambdaApplication<'s>),
    ArrayComp(ArrayComprehension<'s>),
    ObjectComp(ObjectComprehension<'s>),
}

impl std::fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal(l) => write!(f, "{l}"),
            Expression::Array(arr) => {
                write!(f, "[")?;
                for item in arr {
                    match item {
                        ArrayItem::Single(i) => write!(f, "{i},")?,
                        ArrayItem::Spread(i) => write!(f, "...({i}),")?,
                    }
                }
                write!(f, "[")
            }
            Expression::Binary(BinaryExpression {
                operator,
                left,
                right,
            }) => {
                write!(
                    f,
                    "({left} {} {right})",
                    match operator {
                        BinaryOperator::StrictEqual => "==",
                        BinaryOperator::StrictNotEqual => "!=",
                        BinaryOperator::LessThan => "<",
                        BinaryOperator::GreaterThan => ">",
                        BinaryOperator::LessThanEqual => "<=",
                        BinaryOperator::GreaterThanEqual => ">=",
                        BinaryOperator::Plus => "+",
                        BinaryOperator::Minus => "-",
                        BinaryOperator::Times => "*",
                        BinaryOperator::Over => "/",
                        BinaryOperator::Mod => "%",
                        BinaryOperator::In => "in",
                        BinaryOperator::PowerOf => "^",
                        BinaryOperator::Is => "is",
                        BinaryOperator::Cast => "cast",
                    }
                )
            }
            Expression::Identifier(id) => write!(f, "{id}"),
            Expression::Logical(LogicalExpression {
                left,
                right,
                operator,
            }) => {
                write!(
                    f,
                    "({left} {} {right})",
                    match operator {
                        LogicalOperator::Or => "||",
                        LogicalOperator::And => "&&",
                    }
                )
            }
            Expression::Member(MemberExpression { object, property }) => {
                write!(f, "{object}[{property}]")
            }
            Expression::Object(props) => {
                write!(f, "{{")?;
                for prop in props {
                    match prop {
                        ObjectProperty::Single(id) => write!(f, "{id},")?,
                        ObjectProperty::Property(Property { key, value }) => {
                            match key {
                                PropertyKey::Identifier(id) => write!(f, "{id}: {value},"),
                                PropertyKey::Expression(expr) => write!(f, "[{expr}]: {value},"),
                            }?;
                        }
                        ObjectProperty::Spread(expr) => write!(f, "...({expr}),")?,
                    }
                }
                write!(f, "}}")
            }
            Expression::Unary(UnaryExpression { operator, argument }) => {
                write!(
                    f,
                    "({} {argument})",
                    match operator {
                        UnaryOperator::Minus => "-",
                        UnaryOperator::Plus => "+",
                        UnaryOperator::Not => "!",
                    }
                )
            }
            Expression::Call(CallExpression { function, argument }) => {
                write!(f, "{function}({argument})")
            }
            Expression::Template(StringTemplate { parts, suffix }) => {
                write!(f, "$`")?;
                for p in parts {
                    write!(f, "{}${{{}}}", p.fixed_start, p.dynamic_end)?;
                }
                write!(f, "{suffix}`")
            }
            Expression::Abstraction(LambdaAbstraction {arguments, body}) => {
                write!(f, "({arguments}) => {body}")

            },
            Expression::Application(LambdaApplication { lambda, parameter }) => {
                write!(f, "{lambda}({parameter})")
            },
            Expression::ArrayComp(ArrayComprehension{ sources, projection }) => {
                write!(f, "[")?;
                for item in projection {
                    match item {
                        ArrayItem::Single(i) => write!(f, "{i},")?,
                        ArrayItem::Spread(i) => write!(f, "...({i}),")?,
                    }
                }

                for ComprehensionSource{ collection, pattern, predicate } in sources {
                    write!(f, "for {pattern} in {collection}")?;
                    if let Some(p) = predicate {
                        write!(f, " if {p}")?;
                    }
                }

                write!(f, "[")
            },
            Expression::ObjectComp(ObjectComprehension { sources, projection }) => {
                write!(f, "{{")?;
                for prop in projection {
                    match prop {
                        ObjectProperty::Single(id) => write!(f, "{id},")?,
                        ObjectProperty::Property(Property { key, value }) => {
                            match key {
                                PropertyKey::Identifier(id) => write!(f, "{id}: {value},"),
                                PropertyKey::Expression(expr) => write!(f, "[{expr}]: {value},"),
                            }?;
                        }
                        ObjectProperty::Spread(expr) => write!(f, "...({expr}),")?,
                    }
                }

                for ComprehensionSource{ collection, pattern, predicate } in sources {
                    write!(f, "for {pattern} in {collection}")?;
                    if let Some(p) = predicate {
                        write!(f, " if {p}")?;
                    }
                }

                write!(f, "}}")
            },
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ExpressionSet<'s> {
    pub expressions: Vec<Expression<'s>>,
}

type ArrayExpression<'a> = Vec<ArrayItem<'a>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArrayItem<'a> {
    Single(Expression<'a>),
    Spread(Expression<'a>),
}

pub type ObjectExpression<'a> = Vec<ObjectProperty<'a>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectProperty<'a> {
    Single(Identifier<'a>),
    Property(Property<'a>),
    Spread(Expression<'a>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Property<'a> {
    pub key: PropertyKey<'a>,
    pub value: Expression<'a>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyKey<'a> {
    Identifier(Identifier<'a>),
    Expression(Expression<'a>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallExpression<'a> {
    pub function: Identifier<'a>,
    pub argument: Box<Expression<'a>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringTemplate<'a> {
    pub parts: Vec<StringTemplatePart<'a>>,
    pub suffix: Cow<'a, str>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringTemplatePart<'a> {
    pub fixed_start: Cow<'a, str>,
    pub dynamic_end: Box<Expression<'a>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LambdaAbstraction<'a> {
    pub arguments: Pattern<'a>,
    pub body: Box<Expression<'a>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LambdaApplication<'a> {
    pub lambda: Box<Expression<'a>>,
    pub parameter: Box<Expression<'a>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayComprehension<'a> {
    pub sources: Vec<ComprehensionSource<'a>>,
    pub projection: ArrayExpression<'a>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectComprehension<'a> {
    pub sources: Vec<ComprehensionSource<'a>>,
    pub projection: ObjectExpression<'a>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComprehensionSource<'a> {
    pub collection: Box<Expression<'a>>,
    pub pattern: Pattern<'a>,
    pub predicate: Option<Box<Expression<'a>>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnaryExpression<'a> {
    pub operator: UnaryOperator,
    pub argument: Box<Expression<'a>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryExpression<'a> {
    pub operator: BinaryOperator,
    pub left: Box<Expression<'a>>,
    pub right: Box<Expression<'a>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalExpression<'a> {
    pub operator: LogicalOperator,
    pub left: Box<Expression<'a>>,
    pub right: Box<Expression<'a>>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOperator {
    StrictEqual,
    StrictNotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    Plus,
    Minus,
    Times,
    Over,
    Mod,
    In,
    PowerOf,
    Is,
    Cast,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogicalOperator {
    Or,
    And,
}

impl LogicalOperator {
    pub fn short_circuit_on(&self, b: bool) -> bool {
        match self {
            Self::Or => b,
            Self::And => !b,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOperator {
    Minus,
    Plus,
    Not,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberExpression<'a> {
    pub object: Box<Expression<'a>>,
    pub property: Box<Expression<'a>>,
}
