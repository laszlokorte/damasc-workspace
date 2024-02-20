use crate::syntax::expression::IfElseExpression;
use crate::syntax::expression::MatchExpression;
use nom::lib::std::collections::HashSet;
use std::collections::VecDeque;

use either::Either::{self, Left, Right};

use crate::identifier::Identifier;

use super::{
    expression::{
        ArrayComprehension, ArrayItem, BinaryExpression, CallExpression, Expression,
        LambdaAbstraction, LambdaApplication, LogicalExpression, MemberExpression,
        ObjectComprehension, ObjectProperty, Property, PropertyKey, StringTemplate,
        UnaryExpression,
    },
    pattern::{ArrayPatternItem, ObjectPropertyPattern, Pattern, PropertyPattern, Rest},
};

impl Pattern<'_> {
    pub(crate) fn get_identifiers(&self) -> impl Iterator<Item = &Identifier> {
        PatternIterator::new(self).flat_map(|p| match &p {
            Pattern::Capture(id, _) => Either::Left(Some(id).into_iter()),
            Pattern::Identifier(id) => Either::Left(Some(id).into_iter()),
            Pattern::TypedIdentifier(id, _) => Either::Left(Some(id).into_iter()),
            Pattern::Object(props, ..) => Either::Right(props.iter().filter_map(|p| match p {
                ObjectPropertyPattern::Single(id) => Some(id),
                ObjectPropertyPattern::Match(PropertyPattern { key, .. }) => match key {
                    PropertyKey::Identifier(id) => Some(id),
                    PropertyKey::Expression(_expr) => None,
                },
            })),
            Pattern::Discard => Either::Left(None.into_iter()),
            Pattern::TypedDiscard(_) => Either::Left(None.into_iter()),
            Pattern::Literal(_) => Either::Left(None.into_iter()),
            Pattern::Array(_, _) => Either::Left(None.into_iter()),
            Pattern::PinnedExpression(_) => Either::Left(None.into_iter()),
        })
    }

    pub(crate) fn get_expressions(&self) -> impl Iterator<Item = &Expression> {
        PatternIterator::new(self).flat_map(|p| match &p {
            Pattern::Object(props, _) => Either::Left(Box::new(props.iter().filter_map(|p| match p {
                ObjectPropertyPattern::Single(_id) => None,
                ObjectPropertyPattern::Match(PropertyPattern { key, .. }) => match key {
                    PropertyKey::Identifier(_id) => None,
                    PropertyKey::Expression(expr) => Some(expr),
                },
            })) as Box<dyn Iterator<Item = &Expression>>),
            Pattern::Discard => Either::Right(None.into_iter()),
            Pattern::Capture(_, _) => Either::Right(None.into_iter()),
            Pattern::Identifier(_) => Either::Right(None.into_iter()),
            Pattern::TypedDiscard(_) => Either::Right(None.into_iter()),
            Pattern::TypedIdentifier(_, _) => Either::Right(None.into_iter()),
            Pattern::Literal(_) => Either::Right(None.into_iter()),
            Pattern::Array(_, _) => Either::Right(None.into_iter()),
            Pattern::PinnedExpression(e) => Either::Left(Box::new(Some(e.as_ref()).into_iter()) as Box<dyn Iterator<Item = &Expression>>),
        })
    }
}

struct PatternIterator<'e, 's> {
    pattern_stack: VecDeque<&'e Pattern<'s>>,
}

impl<'e, 's> PatternIterator<'e, 's> {
    fn new(pattern: &'e Pattern<'s>) -> Self {
        let mut pattern_stack = VecDeque::new();
        pattern_stack.push_front(pattern);

        Self { pattern_stack }
    }

    fn push_children(&mut self, pattern: &'e Pattern<'s>) {
        match &pattern {
            Pattern::Discard => {}
            Pattern::Capture(_, _) => {}
            Pattern::Identifier(_) => {}
            Pattern::TypedDiscard(_) => {}
            Pattern::PinnedExpression(_) => {}
            Pattern::TypedIdentifier(_, _) => {}
            Pattern::Literal(_) => {}
            Pattern::Object(props, rest) => {
                for p in props {
                    match p {
                        ObjectPropertyPattern::Single(_) => {}
                        ObjectPropertyPattern::Match(PropertyPattern { key, value }) => {
                            match key {
                                PropertyKey::Identifier(_) => {}
                                PropertyKey::Expression(_expr) => {}
                            }
                            self.pattern_stack.push_front(value);
                        }
                    };
                }
                if let Rest::Collect(p) = rest {
                    self.pattern_stack.push_front(p);
                }
            }
            Pattern::Array(items, rest) => {
                for ArrayPatternItem::Pattern(p) in items {
                    self.pattern_stack.push_front(p);
                }
                if let Rest::Collect(p) = rest {
                    self.pattern_stack.push_front(p);
                }
            }
        }
    }
}

impl<'e, 's> Iterator for PatternIterator<'e, 's> {
    type Item = &'e Pattern<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.pattern_stack.pop_front()?;

        self.push_children(next);

        Some(next)
    }
}

struct ExpressionIterator<'e, 's> {
    expression_stack: VecDeque<&'e Expression<'s>>,
    deep: bool,
}

impl<'s, 'e: 's> ExpressionIterator<'e, 's> {
    fn new(expression: &'e Expression<'s>, deep: bool) -> Self {
        let mut expression_stack = VecDeque::new();
        expression_stack.push_front(expression);

        Self {
            expression_stack,
            deep,
        }
    }

    fn push_children(&mut self, expression: &'e Expression<'s>) {
        match expression {
            Expression::Array(arr) => {
                for item in arr {
                    match item {
                        ArrayItem::Single(s) => {
                            self.expression_stack.push_front(s);
                        }
                        ArrayItem::Spread(s) => {
                            self.expression_stack.push_front(s);
                        }
                    }
                }
            }
            Expression::Binary(BinaryExpression { left, right, .. }) => {
                self.expression_stack.push_front(left);
                self.expression_stack.push_front(right);
            }
            Expression::Identifier(_) => {}
            Expression::Literal(_) => {}
            Expression::Logical(LogicalExpression { left, right, .. }) => {
                self.expression_stack.push_front(left);
                self.expression_stack.push_front(right);
            }
            Expression::Member(MemberExpression { object, property }) => {
                self.expression_stack.push_front(object);
                self.expression_stack.push_front(property);
            }
            Expression::Object(props) => {
                for p in props {
                    match p {
                        ObjectProperty::Single(_) => {}
                        ObjectProperty::Property(Property { key, value }) => {
                            self.expression_stack.push_front(value);

                            match key {
                                PropertyKey::Identifier(_id) => {}
                                PropertyKey::Expression(expr) => {
                                    self.expression_stack.push_front(expr)
                                }
                            };
                        }
                        ObjectProperty::Spread(s) => {
                            self.expression_stack.push_front(s);
                        }
                    }
                }
            }
            Expression::Unary(UnaryExpression { argument, .. }) => {
                self.expression_stack.push_front(argument);
            }
            Expression::Call(CallExpression { argument, .. }) => {
                self.expression_stack.push_front(argument);
            }
            Expression::Template(StringTemplate { parts, .. }) => {
                for p in parts {
                    self.expression_stack.push_front(&p.dynamic_end);
                }
            }
            Expression::Abstraction(LambdaAbstraction { arguments, body }) => {
                for expr in arguments.get_expressions() {
                    self.expression_stack.push_front(expr)
                }
                if self.deep {
                    self.expression_stack.push_front(body)
                }
            }
            Expression::Application(LambdaApplication { lambda, parameter }) => {
                self.expression_stack.push_front(lambda);
                self.expression_stack.push_front(parameter);
            }
            Expression::ArrayComp(ArrayComprehension {
                sources,
                projection,
            }) => {
                if self.deep {
                    for src in sources {
                        self.expression_stack.push_front(&src.collection);

                        for expr in src.pattern.get_expressions() {
                            self.expression_stack.push_front(expr);
                        }

                        if let Some(pred) = &src.predicate {
                            self.expression_stack.push_front(pred);
                        }
                    }

                    for item in projection {
                        match item {
                            ArrayItem::Single(expr) => {
                                self.expression_stack.push_front(expr);
                            }
                            ArrayItem::Spread(expr) => {
                                self.expression_stack.push_front(expr);
                            }
                        }
                    }
                }
            }
            Expression::ObjectComp(ObjectComprehension {
                sources,
                projection,
            }) => {
                if self.deep {
                    for src in sources {
                        self.expression_stack.push_front(&src.collection);

                        for expr in src.pattern.get_expressions() {
                            self.expression_stack.push_front(expr);
                        }

                        if let Some(pred) = &src.predicate {
                            self.expression_stack.push_front(pred);
                        }
                    }

                    for item in projection {
                        match item {
                            ObjectProperty::Single(_) => {}
                            ObjectProperty::Property(Property { key, value }) => {
                                self.expression_stack.push_front(value);

                                match key {
                                    PropertyKey::Identifier(_id) => {}
                                    PropertyKey::Expression(expr) => {
                                        self.expression_stack.push_front(expr)
                                    }
                                };
                            }
                            ObjectProperty::Spread(s) => {
                                self.expression_stack.push_front(s);
                            }
                        }
                    }
                }
            }
            Expression::Match(MatchExpression { cases, subject }) => {
                self.expression_stack.push_front(subject);

                for case in cases {
                    for expr in case.pattern.get_expressions() {
                        self.expression_stack.push_front(expr)
                    }
                    if self.deep {
                        self.expression_stack.push_front(&case.body)
                    }
                }
            }
            Expression::Condition(IfElseExpression { condition, true_branch, false_branch }) => {
                self.expression_stack.push_front(condition);
                self.expression_stack.push_front(true_branch);
                if let Some(fb) = false_branch {
                    self.expression_stack.push_front(fb);
                }
            }
        }
    }
}

impl<'s, 'e: 's> Iterator for ExpressionIterator<'e, 's> {
    type Item = &'e Expression<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.expression_stack.pop_front()?;

        self.push_children(next);

        Some(next)
    }
}

impl Expression<'_> {
    pub(crate) fn get_identifiers(&self) -> impl Iterator<Item = &Identifier> {
        ExpressionIterator::new(self, false).flat_map(|e| match e {
            Expression::Object(props) => Left(Box::new(props.iter().filter_map(|p| match p {
                ObjectProperty::Single(id) => Some(id),
                _ => None,
            }))
                as Box<dyn Iterator<Item = &Identifier>>),
            Expression::Identifier(id) => Right(Some(id).into_iter()),
            Expression::Abstraction(LambdaAbstraction { arguments, body }) => {
                let locally_bound = arguments.get_identifiers().collect::<HashSet<_>>();
                let inner_free = body
                    .get_identifiers()
                    .filter(move |v| !locally_bound.contains(v));

                Left(Box::new(inner_free) as Box<dyn Iterator<Item = &Identifier>>)
            }
            Expression::ArrayComp(ArrayComprehension {
                sources,
                projection,
            }) => {
                let (inner_identifiers, locally_bound): (
                    Box<dyn Iterator<Item = &Identifier>>,
                    HashSet<_>,
                ) = sources.iter().fold(
                    (
                        Box::new(std::iter::empty::<&Identifier>()),
                        HashSet::<&Identifier>::new(),
                    ),
                    |(iter, outer_bound), source| {
                        let mut locally_bound = outer_bound.clone();
                        // TODO get rid of those clones
                        let outer_bound1 = outer_bound.clone();
                        let outer_bound2 = outer_bound.clone();
                        locally_bound.extend(source.pattern.get_identifiers());
                        let locally_bound2 = locally_bound.clone();

                        let collection_identifiers = source
                            .collection
                            .get_identifiers()
                            .filter(move |v| !outer_bound1.contains(v));
                        let pattern_identifiers = source
                            .pattern
                            .get_expressions()
                            .flat_map(|e| e.get_identifiers())
                            .filter(move |v| !outer_bound2.contains(v));
                        let predicate_identifiers = source
                            .predicate
                            .iter()
                            .flat_map(|s| s.get_identifiers())
                            .filter(move |v| !locally_bound2.contains(v));

                        let this_level_identifiers = collection_identifiers
                            .chain(predicate_identifiers)
                            .chain(pattern_identifiers);

                        (Box::new(iter.chain(this_level_identifiers)), locally_bound)
                    },
                );

                let projection_identifiers = projection
                    .iter()
                    .flat_map(|p: &ArrayItem| match p {
                        ArrayItem::Single(i) => i.get_identifiers(),
                        ArrayItem::Spread(i) => i.get_identifiers(),
                    })
                    .filter(move |v| !locally_bound.contains(v));

                Left(Box::new(inner_identifiers.chain(projection_identifiers))
                    as Box<dyn Iterator<Item = &Identifier>>)
            }
            Expression::ObjectComp(ObjectComprehension {
                sources,
                projection,
            }) => {
                // TODO refactor duplicate code between ObjectComprehension and ArrayComprehension
                let (inner_identifiers, locally_bound): (
                    Box<dyn Iterator<Item = &Identifier>>,
                    HashSet<_>,
                ) = sources.iter().fold(
                    (
                        Box::new(std::iter::empty::<&Identifier>()),
                        HashSet::<&Identifier>::new(),
                    ),
                    |(iter, outer_bound), source| {
                        let mut locally_bound = outer_bound.clone();
                        // TODO get rid of those clones
                        let outer_bound1 = outer_bound.clone();
                        let outer_bound2 = outer_bound.clone();
                        locally_bound.extend(source.pattern.get_identifiers());
                        let locally_bound2 = locally_bound.clone();

                        let collection_identifiers = source
                            .collection
                            .get_identifiers()
                            .filter(move |v| !outer_bound1.contains(v));
                        let pattern_identifiers = source
                            .pattern
                            .get_expressions()
                            .flat_map(|e| e.get_identifiers())
                            .filter(move |v| !outer_bound2.contains(v));
                        let predicate_identifiers = source
                            .predicate
                            .iter()
                            .flat_map(|s| s.get_identifiers())
                            .filter(move |v| !locally_bound2.contains(v));

                        let this_level_identifiers = collection_identifiers
                            .chain(predicate_identifiers)
                            .chain(pattern_identifiers);

                        (Box::new(iter.chain(this_level_identifiers)), locally_bound)
                    },
                );

                let projection_identifiers = projection
                    .iter()
                    .flat_map(|p: &ObjectProperty| match p {
                        ObjectProperty::Single(id) => {
                            Box::new(std::iter::once(id)) as Box<dyn Iterator<Item = &Identifier>>
                        }
                        ObjectProperty::Property(Property {
                            key: PropertyKey::Identifier(id),
                            value,
                        }) => Box::new(std::iter::once(id).chain(value.get_identifiers()))
                            as Box<dyn Iterator<Item = &Identifier>>,
                        ObjectProperty::Property(Property {
                            key: PropertyKey::Expression(expr),
                            value,
                        }) => Box::new(expr.get_identifiers().chain(value.get_identifiers()))
                            as Box<dyn Iterator<Item = &Identifier>>,
                        ObjectProperty::Spread(expr) => Box::new(expr.get_identifiers())
                            as Box<dyn Iterator<Item = &Identifier>>,
                    })
                    .filter(move |v| !locally_bound.contains(v));

                Left(Box::new(inner_identifiers.chain(projection_identifiers))
                    as Box<dyn Iterator<Item = &Identifier>>)
            }
            Expression::Match(MatchExpression { cases, .. }) => {
                let inner_free = cases.iter().flat_map(|case| {
                    let locally_bound = case.pattern.get_identifiers().collect::<HashSet<_>>();

                    case.body
                        .get_identifiers()
                        .filter(move |v| !locally_bound.contains(v))
                });

                Left(Box::new(inner_free) as Box<dyn Iterator<Item = &Identifier>>)
            }
            _ => Right(None.into_iter()),
        })
    }
}
