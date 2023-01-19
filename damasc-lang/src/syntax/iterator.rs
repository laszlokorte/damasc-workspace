use std::collections::VecDeque;

use either::Either::{self, Right, Left};

use crate::identifier::Identifier;

use super::{pattern::{Pattern, ObjectPropertyPattern, PropertyPattern, ArrayPatternItem, Rest}, expression::{PropertyKey, Expression, ObjectProperty, CallExpression, UnaryExpression, StringTemplate, Property, MemberExpression, LogicalExpression, BinaryExpression, ArrayItem}};


impl Pattern<'_> {
    pub(crate) fn get_identifiers(&self) -> impl Iterator<Item = &Identifier> {
        PatternIterator::new(self).flat_map(|p| {
            match &p {
                Pattern::Capture(id, _) => Either::Left(Some(id).into_iter()),
                Pattern::Identifier(id) => Either::Left(Some(id).into_iter()),
                Pattern::TypedIdentifier(id, _) => Either::Left(Some(id).into_iter()),
                Pattern::Object(props, ..) => {
                    Either::Right(props.into_iter().filter_map(|p| {
                        match p {
                            ObjectPropertyPattern::Single(id) => Some(id),
                            ObjectPropertyPattern::Match(PropertyPattern{key, ..}) => {
                                match key {
                                    PropertyKey::Identifier(id) => Some(id),
                                    PropertyKey::Expression(_expr) => None,
                                }
                            },
                        }
                    }))
                },
                Pattern::Discard => Either::Left(None.into_iter()),
                Pattern::TypedDiscard(_) => Either::Left(None.into_iter()),
                Pattern::Literal(_) => Either::Left(None.into_iter()),
                Pattern::Array(_, _) => Either::Left(None.into_iter()),
            }
        })
    }

    pub(crate) fn get_expressions(&self) -> impl Iterator<Item = &Expression> {
        PatternIterator::new(self).flat_map(|p| {
            match &p {
                Pattern::Object(props, _) => 
                    Either::Left(props.into_iter().filter_map(|p| {
                        match p {
                            ObjectPropertyPattern::Single(_id) => None,
                            ObjectPropertyPattern::Match(PropertyPattern{key, ..}) => {
                                match key {
                                    PropertyKey::Identifier(_id) => None,
                                    PropertyKey::Expression(expr) => Some(expr)
                                }
                            },
                        }
                    })),
                Pattern::Discard => Either::Right(None.into_iter()),
                Pattern::Capture(_, _) => Either::Right(None.into_iter()),
                Pattern::Identifier(_) => Either::Right(None.into_iter()),
                Pattern::TypedDiscard(_) => Either::Right(None.into_iter()),
                Pattern::TypedIdentifier(_, _) => Either::Right(None.into_iter()),
                Pattern::Literal(_) => Either::Right(None.into_iter()),
                Pattern::Array(_, _) => Either::Right(None.into_iter()),
            }
        })
    }
}


struct PatternIterator<'e,'s> {
    pattern_stack : VecDeque<&'e Pattern<'s>>,
}


impl<'e,'s> PatternIterator<'e,'s> {
    fn new(pattern: &'e Pattern<'s>) -> Self {
        let mut pattern_stack = VecDeque::new();
        pattern_stack.push_front(pattern);

        Self {
            pattern_stack
        }
    }

    fn push_children(&mut self, pattern: &'e Pattern<'s>) {
        match &pattern {
            Pattern::Discard => {},
            Pattern::Capture(_, _) => {},
            Pattern::Identifier(_) => {},
            Pattern::TypedDiscard(_) => {},
            Pattern::TypedIdentifier(_, _) => {},
            Pattern::Literal(_) => {},
            Pattern::Object(props, rest) => {
                for p in props {
                    match p {
                        ObjectPropertyPattern::Single(_) => {},
                        ObjectPropertyPattern::Match(PropertyPattern{key, value}) => {
                            match key {
                                PropertyKey::Identifier(_) => {},
                                PropertyKey::Expression(_expr) => {},
                            }
                            self.pattern_stack.push_front(value);
                        },
                    };
                }
                if let Rest::Collect(p) = rest {
                    self.pattern_stack.push_front(p);
                }
            },
            Pattern::Array(items, rest) => {
                for ArrayPatternItem::Pattern(p) in items {
                    self.pattern_stack.push_front(p);
                }
                if let Rest::Collect(p) = rest {
                    self.pattern_stack.push_front(p);
                }
            },
        }
    }
}

impl<'e,'s> Iterator for PatternIterator<'e,'s> {
    type Item = &'e Pattern<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(next) = self.pattern_stack.pop_front() else {
            return None;
        };

        self.push_children(next);

        return Some(next);
    }
    
}


struct ExpressionIterator<'e,'s> {
    expression_stack : VecDeque<&'e Expression<'s>>,
}

impl<'e,'s> ExpressionIterator<'e,'s> {
    fn new(expression: &'e Expression<'s>) -> Self {
        let mut expression_stack = VecDeque::new();
        expression_stack.push_front(expression);

        Self {
            expression_stack
        }
    }

    fn push_children(&mut self, expression: &'e Expression<'s>) {
        match expression {
            Expression::Array(arr) => {
                for item in arr {
                    match item {
                        ArrayItem::Single(s) => {
                            self.expression_stack.push_front(s);
                        },
                        ArrayItem::Spread(s) => {
                            self.expression_stack.push_front(s);
                        },
                    }
                }
            },
            Expression::Binary(BinaryExpression {left, right,..}) => {
                self.expression_stack.push_front(left);
                self.expression_stack.push_front(right);
            },
            Expression::Identifier(_) => {},
            Expression::Literal(_) => {},
            Expression::Logical(LogicalExpression {left, right,..}) => {
                self.expression_stack.push_front(left);
                self.expression_stack.push_front(right);
            },
            Expression::Member(MemberExpression{ object, property }) => {
                self.expression_stack.push_front(object);
                self.expression_stack.push_front(property);
            },
            Expression::Object(props) => {
                for p in props {
                    match p {
                        ObjectProperty::Single(_) => {},
                        ObjectProperty::Property(Property{key, value}) => {
                            self.expression_stack.push_front(value);

                            match key {
                                PropertyKey::Identifier(_id) => {},
                                PropertyKey::Expression(expr) =>
                                self.expression_stack.push_front(expr),
                            };
                        },
                        ObjectProperty::Spread(s) => {
                            self.expression_stack.push_front(s);
                        },
                    }
                }
            },
            Expression::Unary(UnaryExpression{argument, ..}) => {
                self.expression_stack.push_front(argument);
            },
            Expression::Call(CallExpression{argument,..}) => {
                self.expression_stack.push_front(argument);

            },
            Expression::Template(StringTemplate{parts, ..}) => {
                for p in parts {
                    self.expression_stack.push_front(&p.dynamic_end);
                }
            },
        }
    }
}

impl<'e,'s> Iterator for ExpressionIterator<'e,'s> {
    type Item = &'e Expression<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(next) = self.expression_stack.pop_front() else {
            return None;
        };

        self.push_children(next);

        return Some(next);
    }
    
}

impl Expression<'_> {
    pub(crate) fn get_identifiers(&self) -> impl Iterator<Item = &Identifier> {
        ExpressionIterator::new(&self).flat_map(|e| {
            match e {
                Expression::Object(props) => Left(props.into_iter().filter_map(|p| {
                    match p {
                        ObjectProperty::Single(id) => Some(id),
                        _ => None
                    }
                })),
                Expression::Identifier(id) => Right(Some(id).into_iter()),
                _ => Right(None.into_iter()),
            }
        })
    }
}