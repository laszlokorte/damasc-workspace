use crate::syntax::level::EmptyLevel;
use crate::syntax::level::SyntaxLevel;
use std::borrow::Cow;

use crate::identifier::Identifier;
use crate::literal::Literal;

use super::pattern::Pattern;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression<'s, Level: SyntaxLevel = EmptyLevel> {
    Array(ArrayExpression<'s, Level>),
    Binary(BinaryExpression<'s, Level>),
    Identifier(Identifier<'s>),
    Literal(Literal<'s>),
    Logical(LogicalExpression<'s, Level>),
    Member(MemberExpression<'s, Level>),
    Object(ObjectExpression<'s, Level>),
    Unary(UnaryExpression<'s, Level>),
    Call(CallExpression<'s, Level>),
    Template(StringTemplate<'s, Level>),
    Abstraction(LambdaAbstraction<'s, Level>),
    Application(LambdaApplication<'s, Level>),
    ArrayComp(ArrayComprehension<'s, Level>),
    ObjectComp(ObjectComprehension<'s, Level>),
    Condition(IfElseExpression<'s, Level>),
    Match(MatchExpression<'s, Level>),
    Anno(Level::Annotation),
}
impl<'s, Level: SyntaxLevel> Expression<'s, Level> {
    pub(crate) fn deep_clone<'x>(&self) -> Expression<'x, Level> {
        match self {
            Self::Array(a) => Expression::<Level>::Array(a.iter().map(|e| e.deep_clone()).collect()),
            Expression::Binary(b) => Expression::Binary(b.deep_clone()),
            Expression::Identifier(i) => Expression::Identifier(i.deep_clone()),
            Expression::Literal(l) => Expression::Literal(l.deep_clone()),
            Expression::Logical(l) => Expression::Logical(l.deep_clone()),
            Expression::Member(x) => Expression::Member(x.deep_clone()),
            Expression::Object(x) => Expression::Object(x.iter().map(|e| e.deep_clone()).collect()),
            Expression::Unary(x) => Expression::Unary(x.deep_clone()),
            Expression::Call(x) => Expression::Call(x.deep_clone()),
            Expression::Template(x) => Expression::Template(x.deep_clone()),
            Expression::Abstraction(x) => Expression::Abstraction(x.deep_clone()),
            Expression::Application(x) => Expression::Application(x.deep_clone()),
            Expression::ArrayComp(x) => Expression::ArrayComp(x.deep_clone()),
            Expression::ObjectComp(x) => Expression::ObjectComp(x.deep_clone()),
            Expression::Match(x) => Expression::Match(x.deep_clone()),
            Expression::Condition(x) => Expression::Condition(x.deep_clone()),
            Expression::Anno(_) => todo!(),
        }
    }
}

impl<Level: SyntaxLevel> std::fmt::Display for Expression<'_, Level> {
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
            Expression::Abstraction(LambdaAbstraction { arguments, body }) => {
                write!(f, "fn ({arguments}) => {body}")
            }
            Expression::Application(LambdaApplication { lambda, parameter }) => {
                write!(f, "{lambda}({parameter})")
            }
            Expression::Match(MatchExpression { subject, cases }) => {
                write!(f, "match ({subject}) {{")?;
                for case in cases {
                    write!(f, "({0})  => {1}", case.pattern, case.body)?;
                }
                write!(f, "}}")
            }
            Expression::Condition(IfElseExpression { condition, true_branch, false_branch }) => {
                write!(f, "if ({condition}) {{ {true_branch} }}")?;
                if let Some(fb) = false_branch {
                    write!(f, "else {{ {fb} }}")
                } else {
                    Ok(())
                }
            }
            Expression::ArrayComp(ArrayComprehension {
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
            Expression::ObjectComp(ObjectComprehension {
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
            },
            Expression::Anno(_) => todo!()
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ExpressionSet<'s, Level: SyntaxLevel = EmptyLevel> {
    pub expressions: Vec<Expression<'s, Level>>,
}

type ArrayExpression<'a, Level: SyntaxLevel> = Vec<ArrayItem<'a, Level>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArrayItem<'a, Level: SyntaxLevel> {
    Single(Expression<'a, Level>),
    Spread(Expression<'a, Level>),
}
impl<Level: SyntaxLevel> ArrayItem<'_, Level> {
    fn deep_clone<'x>(&self) -> ArrayItem<'x, Level> {
        match self {
            ArrayItem::Single(inner) => ArrayItem::<Level>::Single(inner.deep_clone()),
            ArrayItem::Spread(inner) => ArrayItem::Spread(inner.deep_clone()),
        }
    }
}

pub type ObjectExpression<'a, Level: SyntaxLevel> = Vec<ObjectProperty<'a, Level>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectProperty<'a, Level: SyntaxLevel> {
    Single(Identifier<'a>),
    Property(Property<'a, Level>),
    Spread(Expression<'a, Level>),
}
impl<Level: SyntaxLevel> ObjectProperty<'_, Level> {
    fn deep_clone<'x>(&self) -> ObjectProperty<'x, Level> {
        match self {
            ObjectProperty::Single(i) => ObjectProperty::<Level>::Single(i.deep_clone()),
            ObjectProperty::Property(p) => ObjectProperty::Property(p.deep_clone()),
            ObjectProperty::Spread(e) => ObjectProperty::Spread(e.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Property<'a, Level: SyntaxLevel> {
    pub key: PropertyKey<'a, Level>,
    pub value: Expression<'a, Level>,
}
impl<Level: SyntaxLevel> Property<'_, Level> {
    fn deep_clone<'x>(&self) -> Property<'x, Level> {
        Property::<Level> {
            key: self.key.deep_clone(),
            value: self.value.deep_clone(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyKey<'a, Level: SyntaxLevel> {
    Identifier(Identifier<'a>),
    Expression(Expression<'a, Level>),
}
impl<Level: SyntaxLevel> PropertyKey<'_, Level> {
    pub fn deep_clone<'x>(&self) -> PropertyKey<'x, Level> {
        match self {
            PropertyKey::Identifier(i) => PropertyKey::<Level>::Identifier(i.deep_clone()),
            PropertyKey::Expression(e) => PropertyKey::Expression(e.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallExpression<'a, Level: SyntaxLevel> {
    pub function: Identifier<'a>,
    pub argument: Box<Expression<'a, Level>>,
}
impl<Level: SyntaxLevel> CallExpression<'_, Level> {
    fn deep_clone<'x>(&self) -> CallExpression<'x, Level> {
        CallExpression::<Level> {
            function: self.function.deep_clone(),
            argument: Box::new(self.argument.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringTemplate<'a, Level: SyntaxLevel> {
    pub parts: Vec<StringTemplatePart<'a, Level>>,
    pub suffix: Cow<'a, str>,
}
impl<Level: SyntaxLevel> StringTemplate<'_, Level> {
    fn deep_clone<'x>(&self) -> StringTemplate<'x, Level> {
        StringTemplate::<Level> {
            parts: self.parts.iter().map(|p| p.deep_clone()).collect(),
            suffix: Cow::Owned(self.suffix.to_string()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringTemplatePart<'a, Level: SyntaxLevel> {
    pub fixed_start: Cow<'a, str>,
    pub dynamic_end: Box<Expression<'a, Level>>,
}
impl<Level: SyntaxLevel> StringTemplatePart<'_, Level> {
    fn deep_clone<'x>(&self) -> StringTemplatePart<'x, Level> {
        StringTemplatePart::<Level> {
            fixed_start: Cow::Owned(self.fixed_start.to_string()),
            dynamic_end: Box::new(self.dynamic_end.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LambdaAbstraction<'a, Level: SyntaxLevel> {
    pub arguments: Pattern<'a, Level>,
    pub body: Box<Expression<'a, Level>>,
}
impl<Level: SyntaxLevel> LambdaAbstraction<'_, Level> {
    fn deep_clone<'x>(&self) -> LambdaAbstraction<'x, Level> {
        LambdaAbstraction::<Level> {
            arguments: self.arguments.deep_clone(),
            body: Box::new(self.body.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LambdaApplication<'a, Level: SyntaxLevel> {
    pub lambda: Box<Expression<'a, Level>>,
    pub parameter: Box<Expression<'a, Level>>,
}
impl<Level: SyntaxLevel> LambdaApplication<'_, Level> {
    fn deep_clone<'x>(&self) -> LambdaApplication<'x, Level> {
        LambdaApplication::<Level> {
            lambda: Box::new(self.lambda.deep_clone()),
            parameter: Box::new(self.parameter.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MatchCase<'a, Level: SyntaxLevel> {
    pub pattern: Pattern<'a, Level>,
    pub guard: Option<Box<Expression<'a, Level>>>,
    pub body: Box<Expression<'a, Level>>,
}

impl<Level: SyntaxLevel> MatchCase<'_, Level> {
    fn deep_clone<'x>(&self) -> MatchCase<'x, Level> {
        MatchCase::<Level> {
            pattern: self.pattern.deep_clone(),
            guard: self.guard.clone().map(|g| Box::new(g.deep_clone())),
            body: Box::new(self.body.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MatchExpression<'a, Level: SyntaxLevel> {
    pub subject: Box<Expression<'a, Level>>,
    pub cases: Vec<MatchCase<'a, Level>>,
}

impl<Level: SyntaxLevel> MatchExpression<'_, Level> {
    fn deep_clone<'x>(&self) -> MatchExpression<'x, Level> {
        MatchExpression::<Level> {
            subject: Box::new(self.subject.deep_clone()),
            cases: self.cases.iter().map(|p| p.deep_clone()).collect(),
        }
    }
}



#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IfElseExpression<'a, Level: SyntaxLevel> {
    pub condition: Box<Expression<'a, Level>>,
    pub true_branch: Box<Expression<'a, Level>>,
    pub false_branch: Option<Box<Expression<'a, Level>>>,
}

impl<Level: SyntaxLevel> IfElseExpression<'_, Level> {
    fn deep_clone<'x>(&self) -> IfElseExpression<'x, Level> {
        IfElseExpression::<Level> {
            condition: Box::new(self.condition.deep_clone()),
            true_branch: Box::new(self.true_branch.deep_clone()),
            false_branch: self.false_branch.clone().map(|fb| Box::new(fb.deep_clone())),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayComprehension<'a, Level: SyntaxLevel> {
    pub sources: Vec<ComprehensionSource<'a, Level>>,
    pub projection: ArrayExpression<'a, Level>,
}
impl<Level: SyntaxLevel> ArrayComprehension<'_, Level> {
    fn deep_clone<'x>(&self) -> ArrayComprehension<'x, Level> {
        ArrayComprehension::<Level> {
            sources: self.sources.iter().map(|e| e.deep_clone()).collect(),
            projection: self.projection.iter().map(|e| e.deep_clone()).collect(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectComprehension<'a, Level: SyntaxLevel> {
    pub sources: Vec<ComprehensionSource<'a, Level>>,
    pub projection: ObjectExpression<'a, Level>,
}
impl<Level: SyntaxLevel> ObjectComprehension<'_, Level> {
    fn deep_clone<'x>(&self) -> ObjectComprehension<'x, Level> {
        ObjectComprehension::<Level> {
            sources: self.sources.iter().map(|e| e.deep_clone()).collect(),
            projection: self.projection.iter().map(|e| e.deep_clone()).collect(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComprehensionSource<'a, Level: SyntaxLevel> {
    pub collection: Box<Expression<'a, Level>>,
    pub pattern: Pattern<'a, Level>,
    pub strong_pattern: bool,
    pub predicate: Option<Box<Expression<'a, Level>>>,
}
impl<Level: SyntaxLevel> ComprehensionSource<'_, Level> {
    fn deep_clone<'x>(&self) -> ComprehensionSource<'x, Level> {
        ComprehensionSource::<Level> {
            collection: Box::new(self.collection.deep_clone()),
            pattern: self.pattern.deep_clone(),
            strong_pattern: self.strong_pattern,
            predicate: self.predicate.clone().map(|b| Box::new(b.deep_clone())),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnaryExpression<'a, Level: SyntaxLevel> {
    pub operator: UnaryOperator,
    pub argument: Box<Expression<'a, Level>>,
}
impl<Level: SyntaxLevel> UnaryExpression<'_, Level> {
    fn deep_clone<'x>(&self) -> UnaryExpression<'x, Level> {
        UnaryExpression::<Level> {
            operator: self.operator,
            argument: Box::new(self.argument.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryExpression<'a, Level: SyntaxLevel> {
    pub operator: BinaryOperator,
    pub left: Box<Expression<'a, Level>>,
    pub right: Box<Expression<'a, Level>>,
}
impl<Level: SyntaxLevel> BinaryExpression<'_, Level> {
    fn deep_clone<'x>(&self) -> BinaryExpression<'x, Level> {
        BinaryExpression::<Level> {
            operator: self.operator,
            left: Box::new(self.left.deep_clone()),
            right: Box::new(self.right.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalExpression<'a, Level: SyntaxLevel> {
    pub operator: LogicalOperator,
    pub left: Box<Expression<'a, Level>>,
    pub right: Box<Expression<'a, Level>>,
}
impl<Level: SyntaxLevel> LogicalExpression<'_, Level> {
    fn deep_clone<'x>(&self) -> LogicalExpression<'x, Level> {
        LogicalExpression::<Level> {
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
pub struct MemberExpression<'a, Level: SyntaxLevel> {
    pub object: Box<Expression<'a, Level>>,
    pub property: Box<Expression<'a, Level>>,
}
impl<Level: SyntaxLevel> MemberExpression<'_, Level> {
    fn deep_clone<'x>(&self) -> MemberExpression<'x, Level> {
        MemberExpression::<Level> {
            object: Box::new(self.object.deep_clone()),
            property: Box::new(self.property.deep_clone()),
        }
    }
}
