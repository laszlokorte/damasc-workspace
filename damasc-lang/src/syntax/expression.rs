use crate::syntax::pattern::AnnotatedPattern;
use std::borrow::Cow;

use crate::identifier::Identifier;
use crate::literal::Literal;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnnotatedExpression<'s, Annotation> {
    pub body: Expression<'s, Annotation>,
    pub annotation: Annotation,
}

impl<'s, Annotation:Clone> AnnotatedExpression<'s, Annotation> {
    pub(crate) fn deep_clone<'x>(&self) -> AnnotatedExpression<'x, Annotation> {
        AnnotatedExpression {
            body: self.body.deep_clone(),
            annotation: self.annotation.clone(),
        }
    }
}


impl<Annotation> std::fmt::Display for AnnotatedExpression<'_, Annotation> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}", self.body)
    }

}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnnotatedIdentifier<'s, Annotation> {
    pub body: Identifier<'s>,
    pub annotation: Annotation,
}

impl<'s, Annotation:Clone> AnnotatedIdentifier<'s, Annotation> {
    pub(crate) fn deep_clone<'x>(&self) -> AnnotatedIdentifier<'x, Annotation> {
        AnnotatedIdentifier {
            body: self.body.deep_clone(),
            annotation: self.annotation.clone(),
        }
    }
}


impl<Annotation> std::fmt::Display for AnnotatedIdentifier<'_, Annotation> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}", self.body)
    }

}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnnotatedLiteral<'s, Annotation> {
    pub body: Literal<'s>,
    pub annotation: Annotation,
}
impl<'s, Annotation:Clone> AnnotatedLiteral<'s, Annotation> {
    pub(crate) fn deep_clone<'x>(&self) -> AnnotatedLiteral<'x, Annotation> {
        AnnotatedLiteral {
            body: self.body.deep_clone(),
            annotation: self.annotation.clone(),
        }
    }
}



impl<Annotation> std::fmt::Display for AnnotatedLiteral<'_, Annotation> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}", self.body)
    }

}

pub type BoxedExpression<'s, Annotation> = Box<AnnotatedExpression<'s, Annotation>>;


#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression<'s, Annotation> {
    Array(ArrayExpression<'s, Annotation>),
    Binary(BinaryExpression<'s, Annotation>),
    Identifier(AnnotatedIdentifier<'s, Annotation>),
    Literal(AnnotatedLiteral<'s, Annotation>),
    Logical(LogicalExpression<'s, Annotation>),
    Member(MemberExpression<'s, Annotation>),
    Object(ObjectExpression<'s, Annotation>),
    Unary(UnaryExpression<'s, Annotation>),
    Call(CallExpression<'s, Annotation>),
    Template(StringTemplate<'s, Annotation>),
    Abstraction(LambdaAbstraction<'s, Annotation>),
    Application(LambdaApplication<'s, Annotation>),
    ArrayComp(ArrayComprehension<'s, Annotation>),
    ObjectComp(ObjectComprehension<'s, Annotation>),
    Condition(IfElseExpression<'s, Annotation>),
    Match(MatchExpression<'s, Annotation>),
}

impl<'s, Annotation:Clone> Expression<'s, Annotation> {
    pub(crate) fn deep_clone<'x>(&self) -> Expression<'x, Annotation> {
        match self {
            Self::Array(a) => Expression::Array(a.iter().map(|e| e.deep_clone()).collect()),
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
        }
    }
}

impl<Annotation> std::fmt::Display for Expression<'_, Annotation> {
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
            }
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ExpressionSet<'s, Annotation> {
    pub expressions: Vec<Expression<'s, Annotation>>,
}

type ArrayExpression<'a, Annotation> = Vec<ArrayItem<'a, Annotation>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArrayItem<'a, Annotation> {
    Single(AnnotatedExpression<'a, Annotation>),
    Spread(AnnotatedExpression<'a, Annotation>),
}
impl<Annotation:Clone> ArrayItem<'_, Annotation> {
    fn deep_clone<'x>(&self) -> ArrayItem<'x, Annotation> {
        match self {
            ArrayItem::Single(inner) => ArrayItem::Single(inner.deep_clone()),
            ArrayItem::Spread(inner) => ArrayItem::Spread(inner.deep_clone()),
        }
    }
}

pub type ObjectExpression<'a, Annotation> = Vec<ObjectProperty<'a, Annotation>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectProperty<'a, Annotation> {
    Single(AnnotatedIdentifier<'a, Annotation>),
    Property(Property<'a, Annotation>),
    Spread(AnnotatedExpression<'a, Annotation>),
}
impl<Annotation:Clone> ObjectProperty<'_, Annotation> {
    fn deep_clone<'x>(&self) -> ObjectProperty<'x, Annotation> {
        match self {
            ObjectProperty::Single(i) => ObjectProperty::Single(i.deep_clone()),
            ObjectProperty::Property(p) => ObjectProperty::Property(p.deep_clone()),
            ObjectProperty::Spread(e) => ObjectProperty::Spread(e.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Property<'a, Annotation> {
    pub key: PropertyKey<'a, Annotation>,
    pub value: AnnotatedExpression<'a, Annotation>,
}
impl<Annotation:Clone> Property<'_, Annotation> {
    fn deep_clone<'x>(&self) -> Property<'x, Annotation> {
        Property {
            key: self.key.deep_clone(),
            value: self.value.deep_clone(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyKey<'a, Annotation> {
    Identifier(AnnotatedIdentifier<'a, Annotation>),
    Expression(AnnotatedExpression<'a, Annotation>),
}
impl<Annotation:Clone> PropertyKey<'_, Annotation> {
    pub fn deep_clone<'x>(&self) -> PropertyKey<'x, Annotation> {
        match self {
            PropertyKey::Identifier(i) => PropertyKey::Identifier(i.deep_clone()),
            PropertyKey::Expression(e) => PropertyKey::Expression(e.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallExpression<'a, Annotation> {
    pub function: AnnotatedIdentifier<'a, Annotation>,
    pub argument: BoxedExpression<'a, Annotation>,
}
impl<Annotation:Clone> CallExpression<'_, Annotation> {
    fn deep_clone<'x>(&self) -> CallExpression<'x, Annotation> {
        CallExpression {
            function: self.function.deep_clone(),
            argument: Box::new(self.argument.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringTemplate<'a, Annotation> {
    pub parts: Vec<StringTemplatePart<'a, Annotation>>,
    pub suffix: Cow<'a, str>,
}
impl<Annotation:Clone> StringTemplate<'_, Annotation> {
    fn deep_clone<'x>(&self) -> StringTemplate<'x, Annotation> {
        StringTemplate {
            parts: self.parts.iter().map(|p| p.deep_clone()).collect(),
            suffix: Cow::Owned(self.suffix.to_string()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringTemplatePart<'a,Annotation> {
    pub fixed_start: Cow<'a, str>,
    pub dynamic_end: BoxedExpression<'a,Annotation>,
}
impl<Annotation:Clone> StringTemplatePart<'_, Annotation> {
    fn deep_clone<'x>(&self) -> StringTemplatePart<'x, Annotation> {
        StringTemplatePart {
            fixed_start: Cow::Owned(self.fixed_start.to_string()),
            dynamic_end: Box::new(self.dynamic_end.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LambdaAbstraction<'a,Annotation> {
    pub arguments: AnnotatedPattern<'a, Annotation>,
    pub body: BoxedExpression<'a, Annotation>,
}
impl<Annotation:Clone> LambdaAbstraction<'_, Annotation> {
    fn deep_clone<'x>(&self) -> LambdaAbstraction<'x, Annotation> {
        LambdaAbstraction {
            arguments: self.arguments.deep_clone(),
            body: Box::new(self.body.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LambdaApplication<'a, Annotation> {
    pub lambda: BoxedExpression<'a, Annotation>,
    pub parameter: BoxedExpression<'a, Annotation>,
}
impl<Annotation:Clone> LambdaApplication<'_, Annotation> {
    fn deep_clone<'x>(&self) -> LambdaApplication<'x, Annotation> {
        LambdaApplication {
            lambda: Box::new(self.lambda.deep_clone()),
            parameter: Box::new(self.parameter.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MatchCase<'a, Annotation> {
    pub pattern: AnnotatedPattern<'a, Annotation>,
    pub guard: Option<BoxedExpression<'a, Annotation>>,
    pub body: BoxedExpression<'a, Annotation>,
}

impl<Annotation:Clone> MatchCase<'_, Annotation> {
    fn deep_clone<'x>(&self) -> MatchCase<'x, Annotation> {
        MatchCase {
            pattern: self.pattern.deep_clone(),
            guard: self.guard.clone().map(|g| Box::new(g.deep_clone())),
            body: Box::new(self.body.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MatchExpression<'a, Annotation> {
    pub subject: BoxedExpression<'a, Annotation>,
    pub cases: Vec<MatchCase<'a, Annotation>>,
}

impl<Annotation:Clone> MatchExpression<'_, Annotation> {
    fn deep_clone<'x>(&self) -> MatchExpression<'x, Annotation> {
        MatchExpression {
            subject: Box::new(self.subject.deep_clone()),
            cases: self.cases.iter().map(|p| p.deep_clone()).collect(),
        }
    }
}



#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IfElseExpression<'a, Annotation> {
    pub condition: BoxedExpression<'a, Annotation>,
    pub true_branch: BoxedExpression<'a, Annotation>,
    pub false_branch: Option<BoxedExpression<'a, Annotation>>,
}

impl<Annotation:Clone> IfElseExpression<'_, Annotation> {
    fn deep_clone<'x>(&self) -> IfElseExpression<'x, Annotation> {
        IfElseExpression {
            condition: Box::new(self.condition.deep_clone()),
            true_branch: Box::new(self.true_branch.deep_clone()),
            false_branch: self.false_branch.clone().map(|fb| Box::new(fb.deep_clone())),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayComprehension<'a, Annotation> {
    pub sources: Vec<ComprehensionSource<'a, Annotation>>,
    pub projection: ArrayExpression<'a, Annotation>,
}
impl<Annotation:Clone> ArrayComprehension<'_, Annotation> {
    fn deep_clone<'x>(&self) -> ArrayComprehension<'x, Annotation> {
        ArrayComprehension {
            sources: self.sources.iter().map(|e| e.deep_clone()).collect(),
            projection: self.projection.iter().map(|e| e.deep_clone()).collect(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectComprehension<'a, Annotation> {
    pub sources: Vec<ComprehensionSource<'a, Annotation>>,
    pub projection: ObjectExpression<'a, Annotation>,
}
impl<Annotation:Clone> ObjectComprehension<'_, Annotation> {
    fn deep_clone<'x>(&self) -> ObjectComprehension<'x, Annotation> {
        ObjectComprehension {
            sources: self.sources.iter().map(|e| e.deep_clone()).collect(),
            projection: self.projection.iter().map(|e| e.deep_clone()).collect(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComprehensionSource<'a, Annotation> {
    pub collection: BoxedExpression<'a, Annotation>,
    pub pattern: AnnotatedPattern<'a, Annotation>,
    pub strong_pattern: bool,
    pub predicate: Option<BoxedExpression<'a, Annotation>>,
}
impl<Annotation:Clone> ComprehensionSource<'_, Annotation> {
    fn deep_clone<'x>(&self) -> ComprehensionSource<'x, Annotation> {
        ComprehensionSource {
            collection: Box::new(self.collection.deep_clone()),
            pattern: self.pattern.deep_clone(),
            strong_pattern: self.strong_pattern,
            predicate: self.predicate.clone().map(|b| Box::new(b.deep_clone())),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnaryExpression<'a, Annotation> {
    pub operator: UnaryOperator,
    pub argument: BoxedExpression<'a, Annotation>,
}
impl<Annotation:Clone> UnaryExpression<'_, Annotation> {
    fn deep_clone<'x>(&self) -> UnaryExpression<'x, Annotation> {
        UnaryExpression {
            operator: self.operator,
            argument: Box::new(self.argument.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryExpression<'a, Annotation> {
    pub operator: BinaryOperator,
    pub left: BoxedExpression<'a, Annotation>,
    pub right: BoxedExpression<'a, Annotation>,
}
impl<Annotation:Clone> BinaryExpression<'_, Annotation> {
    fn deep_clone<'x>(&self) -> BinaryExpression<'x, Annotation> {
        BinaryExpression {
            operator: self.operator,
            left: Box::new(self.left.deep_clone()),
            right: Box::new(self.right.deep_clone()),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalExpression<'a, Annotation> {
    pub operator: LogicalOperator,
    pub left: BoxedExpression<'a, Annotation>,
    pub right: BoxedExpression<'a, Annotation>,
}
impl<Annotation: Clone> LogicalExpression<'_, Annotation> {
    fn deep_clone<'x>(&self) -> LogicalExpression<'x, Annotation> {
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
pub struct MemberExpression<'a, Annotation> {
    pub object: BoxedExpression<'a, Annotation>,
    pub property: BoxedExpression<'a, Annotation>,
}
impl<Annotation:Clone> MemberExpression<'_, Annotation> {
    fn deep_clone<'x>(&self) -> MemberExpression<'x, Annotation> {
        MemberExpression {
            object: Box::new(self.object.deep_clone()),
            property: Box::new(self.property.deep_clone()),
        }
    }
}
