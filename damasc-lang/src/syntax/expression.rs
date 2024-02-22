use crate::syntax::location::Location;
use std::borrow::Cow;

use crate::identifier::Identifier;
use crate::literal::Literal;

use super::pattern::Pattern;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expression<'s> {
    pub body: ExpressionBody<'s>,
    pub location: Option<Location>,
}

impl<'s> Expression<'s> {
    pub fn new(body: ExpressionBody<'s>) -> Expression<'s> {
        Self {
            body,
            location: None,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExpressionBody<'s> {
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
    Condition(IfElseExpression<'s>),
    Match(MatchExpression<'s>),
}

impl<'s> Expression<'s> {
    pub(crate) fn deep_clone<'x>(&self) -> Expression<'x> {
        Expression::new(match &self.body {
            ExpressionBody::Array(a) => {
                ExpressionBody::Array(a.iter().map(|e| e.deep_clone()).collect())
            }
            ExpressionBody::Binary(b) => ExpressionBody::Binary(b.deep_clone()),
            ExpressionBody::Identifier(i) => ExpressionBody::Identifier(i.deep_clone()),
            ExpressionBody::Literal(l) => ExpressionBody::Literal(l.deep_clone()),
            ExpressionBody::Logical(l) => ExpressionBody::Logical(l.deep_clone()),
            ExpressionBody::Member(x) => ExpressionBody::Member(x.deep_clone()),
            ExpressionBody::Object(x) => {
                ExpressionBody::Object(x.iter().map(|e| e.deep_clone()).collect())
            }
            ExpressionBody::Unary(x) => ExpressionBody::Unary(x.deep_clone()),
            ExpressionBody::Call(x) => ExpressionBody::Call(x.deep_clone()),
            ExpressionBody::Template(x) => ExpressionBody::Template(x.deep_clone()),
            ExpressionBody::Abstraction(x) => ExpressionBody::Abstraction(x.deep_clone()),
            ExpressionBody::Application(x) => ExpressionBody::Application(x.deep_clone()),
            ExpressionBody::ArrayComp(x) => ExpressionBody::ArrayComp(x.deep_clone()),
            ExpressionBody::ObjectComp(x) => ExpressionBody::ObjectComp(x.deep_clone()),
            ExpressionBody::Match(x) => ExpressionBody::Match(x.deep_clone()),
            ExpressionBody::Condition(x) => ExpressionBody::Condition(x.deep_clone()),
        })
    }
}

impl std::fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.body {
            ExpressionBody::Literal(l) => write!(f, "{l}"),
            ExpressionBody::Array(arr) => {
                write!(f, "[")?;
                for item in arr {
                    match item {
                        ArrayItem::Single(i) => write!(f, "{i},")?,
                        ArrayItem::Spread(i) => write!(f, "...({i}),")?,
                    }
                }
                write!(f, "[")
            }
            ExpressionBody::Binary(BinaryExpression {
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
            ExpressionBody::Identifier(id) => write!(f, "{id}"),
            ExpressionBody::Logical(LogicalExpression {
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
            ExpressionBody::Member(MemberExpression { object, property }) => {
                write!(f, "{object}[{property}]")
            }
            ExpressionBody::Object(props) => {
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
            ExpressionBody::Unary(UnaryExpression { operator, argument }) => {
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
            ExpressionBody::Call(CallExpression { function, argument }) => {
                write!(f, "{function}({argument})")
            }
            ExpressionBody::Template(StringTemplate { parts, suffix }) => {
                write!(f, "$`")?;
                for p in parts {
                    write!(f, "{}${{{}}}", p.fixed_start, p.dynamic_end)?;
                }
                write!(f, "{suffix}`")
            }
            ExpressionBody::Abstraction(LambdaAbstraction { arguments, body }) => {
                write!(f, "fn ({arguments}) => {body}")
            }
            ExpressionBody::Application(LambdaApplication { lambda, parameter }) => {
                write!(f, "{lambda}({parameter})")
            }
            ExpressionBody::Match(MatchExpression { subject, cases }) => {
                write!(f, "match ({subject}) {{")?;
                for case in cases {
                    write!(f, "({0})  => {1}", case.pattern, case.body)?;
                }
                write!(f, "}}")
            }
            ExpressionBody::Condition(IfElseExpression {
                condition,
                true_branch,
                false_branch,
            }) => {
                write!(f, "if ({condition}) {{ {true_branch} }}")?;
                if let Some(fb) = false_branch {
                    write!(f, "else {{ {fb} }}")
                } else {
                    Ok(())
                }
            }
            ExpressionBody::ArrayComp(ArrayComprehension {
                sources,
                projection,
            }) => {
                write!(f, "[")?;
                for item in projection {
                    match item {
                        ArrayItem::Single(i) => write!(f, "{i},")?,
                        ArrayItem::Spread(i) => write!(f, "...({i}),")?,
                    }
                }

                for ComprehensionSource {
                    collection,
                    pattern,
                    predicate,
                    strong_pattern,
                } in sources
                {
                    if *strong_pattern {
                        write!(f, "for {pattern} in {collection}")?;
                    } else {
                        write!(f, "for match {pattern} in {collection}")?;
                    }
                    if let Some(p) = predicate {
                        write!(f, " if {p}")?;
                    }
                }

                write!(f, "[")
            }
            ExpressionBody::ObjectComp(ObjectComprehension {
                sources,
                projection,
            }) => {
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

                for ComprehensionSource {
                    collection,
                    pattern,
                    predicate,
                    strong_pattern,
                } in sources
                {
                    if *strong_pattern {
                        write!(f, "for {pattern} in {collection}")?;
                    } else {
                        write!(f, "for match {pattern} in {collection}")?;
                    }
                    if let Some(p) = predicate {
                        write!(f, " if {p}")?;
                    }
                }

                write!(f, "}}")
            }
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
impl ArrayItem<'_> {
    fn deep_clone<'x>(&self) -> ArrayItem<'x> {
        match self {
            ArrayItem::Single(inner) => ArrayItem::Single(inner.deep_clone()),
            ArrayItem::Spread(inner) => ArrayItem::Spread(inner.deep_clone()),
        }
    }
}

pub type ObjectExpression<'a> = Vec<ObjectProperty<'a>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectProperty<'a> {
    Single(Identifier<'a>),
    Property(Property<'a>),
    Spread(Expression<'a>),
}
impl ObjectProperty<'_> {
    fn deep_clone<'x>(&self) -> ObjectProperty<'x> {
        match self {
            ObjectProperty::Single(i) => ObjectProperty::Single(i.deep_clone()),
            ObjectProperty::Property(p) => ObjectProperty::Property(p.deep_clone()),
            ObjectProperty::Spread(e) => ObjectProperty::Spread(e.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Property<'a> {
    pub key: PropertyKey<'a>,
    pub value: Expression<'a>,
}
impl Property<'_> {
    fn deep_clone<'x>(&self) -> Property<'x> {
        Property {
            key: self.key.deep_clone(),
            value: self.value.deep_clone(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyKey<'a> {
    Identifier(Identifier<'a>),
    Expression(Expression<'a>),
}
impl PropertyKey<'_> {
    pub fn deep_clone<'x>(&self) -> PropertyKey<'x> {
        match self {
            PropertyKey::Identifier(i) => PropertyKey::Identifier(i.deep_clone()),
            PropertyKey::Expression(e) => PropertyKey::Expression(e.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallExpression<'a> {
    pub function: Identifier<'a>,
    pub argument: Box<Expression<'a>>,
}
impl CallExpression<'_> {
    fn deep_clone<'x>(&self) -> CallExpression<'x> {
        CallExpression {
            function: self.function.deep_clone(),
            argument: Box::new(self.argument.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringTemplate<'a> {
    pub parts: Vec<StringTemplatePart<'a>>,
    pub suffix: Cow<'a, str>,
}
impl StringTemplate<'_> {
    fn deep_clone<'x>(&self) -> StringTemplate<'x> {
        StringTemplate {
            parts: self.parts.iter().map(|p| p.deep_clone()).collect(),
            suffix: Cow::Owned(self.suffix.to_string()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringTemplatePart<'a> {
    pub fixed_start: Cow<'a, str>,
    pub dynamic_end: Box<Expression<'a>>,
}
impl StringTemplatePart<'_> {
    fn deep_clone<'x>(&self) -> StringTemplatePart<'x> {
        StringTemplatePart {
            fixed_start: Cow::Owned(self.fixed_start.to_string()),
            dynamic_end: Box::new(self.dynamic_end.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LambdaAbstraction<'a> {
    pub arguments: Pattern<'a>,
    pub body: Box<Expression<'a>>,
}
impl LambdaAbstraction<'_> {
    fn deep_clone<'x>(&self) -> LambdaAbstraction<'x> {
        LambdaAbstraction {
            arguments: self.arguments.deep_clone(),
            body: Box::new(self.body.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LambdaApplication<'a> {
    pub lambda: Box<Expression<'a>>,
    pub parameter: Box<Expression<'a>>,
}
impl LambdaApplication<'_> {
    fn deep_clone<'x>(&self) -> LambdaApplication<'x> {
        LambdaApplication {
            lambda: Box::new(self.lambda.deep_clone()),
            parameter: Box::new(self.parameter.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MatchCase<'a> {
    pub pattern: Pattern<'a>,
    pub guard: Option<Box<Expression<'a>>>,
    pub body: Box<Expression<'a>>,
}

impl MatchCase<'_> {
    fn deep_clone<'x>(&self) -> MatchCase<'x> {
        MatchCase {
            pattern: self.pattern.deep_clone(),
            guard: self.guard.clone().map(|g| Box::new(g.deep_clone())),
            body: Box::new(self.body.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MatchExpression<'a> {
    pub subject: Box<Expression<'a>>,
    pub cases: Vec<MatchCase<'a>>,
}

impl MatchExpression<'_> {
    fn deep_clone<'x>(&self) -> MatchExpression<'x> {
        MatchExpression {
            subject: Box::new(self.subject.deep_clone()),
            cases: self.cases.iter().map(|p| p.deep_clone()).collect(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IfElseExpression<'a> {
    pub condition: Box<Expression<'a>>,
    pub true_branch: Box<Expression<'a>>,
    pub false_branch: Option<Box<Expression<'a>>>,
}

impl IfElseExpression<'_> {
    fn deep_clone<'x>(&self) -> IfElseExpression<'x> {
        IfElseExpression {
            condition: Box::new(self.condition.deep_clone()),
            true_branch: Box::new(self.true_branch.deep_clone()),
            false_branch: self
                .false_branch
                .clone()
                .map(|fb| Box::new(fb.deep_clone())),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayComprehension<'a> {
    pub sources: Vec<ComprehensionSource<'a>>,
    pub projection: ArrayExpression<'a>,
}
impl ArrayComprehension<'_> {
    fn deep_clone<'x>(&self) -> ArrayComprehension<'x> {
        ArrayComprehension {
            sources: self.sources.iter().map(|e| e.deep_clone()).collect(),
            projection: self.projection.iter().map(|e| e.deep_clone()).collect(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectComprehension<'a> {
    pub sources: Vec<ComprehensionSource<'a>>,
    pub projection: ObjectExpression<'a>,
}
impl ObjectComprehension<'_> {
    fn deep_clone<'x>(&self) -> ObjectComprehension<'x> {
        ObjectComprehension {
            sources: self.sources.iter().map(|e| e.deep_clone()).collect(),
            projection: self.projection.iter().map(|e| e.deep_clone()).collect(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComprehensionSource<'a> {
    pub collection: Box<Expression<'a>>,
    pub pattern: Pattern<'a>,
    pub strong_pattern: bool,
    pub predicate: Option<Box<Expression<'a>>>,
}
impl ComprehensionSource<'_> {
    fn deep_clone<'x>(&self) -> ComprehensionSource<'x> {
        ComprehensionSource {
            collection: Box::new(self.collection.deep_clone()),
            pattern: self.pattern.deep_clone(),
            strong_pattern: self.strong_pattern,
            predicate: self.predicate.clone().map(|b| Box::new(b.deep_clone())),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnaryExpression<'a> {
    pub operator: UnaryOperator,
    pub argument: Box<Expression<'a>>,
}
impl UnaryExpression<'_> {
    fn deep_clone<'x>(&self) -> UnaryExpression<'x> {
        UnaryExpression {
            operator: self.operator,
            argument: Box::new(self.argument.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryExpression<'a> {
    pub operator: BinaryOperator,
    pub left: Box<Expression<'a>>,
    pub right: Box<Expression<'a>>,
}
impl BinaryExpression<'_> {
    fn deep_clone<'x>(&self) -> BinaryExpression<'x> {
        BinaryExpression {
            operator: self.operator,
            left: Box::new(self.left.deep_clone()),
            right: Box::new(self.right.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalExpression<'a> {
    pub operator: LogicalOperator,
    pub left: Box<Expression<'a>>,
    pub right: Box<Expression<'a>>,
}
impl LogicalExpression<'_> {
    fn deep_clone<'x>(&self) -> LogicalExpression<'x> {
        LogicalExpression {
            operator: self.operator,
            left: Box::new(self.left.deep_clone()),
            right: Box::new(self.right.deep_clone()),
        }
    }
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
impl MemberExpression<'_> {
    fn deep_clone<'x>(&self) -> MemberExpression<'x> {
        MemberExpression {
            object: Box::new(self.object.deep_clone()),
            property: Box::new(self.property.deep_clone()),
        }
    }
}
