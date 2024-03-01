use crate::runtime::evaluation::EvalError;
use crate::syntax::location::Location;
use crate::syntax::pattern::PatternBody;
use crate::value_type::ValueType;

use std::{
    borrow::Cow,
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
};

use crate::{
    identifier::Identifier,
    literal::Literal,
    syntax::{
        expression::PropertyKey,
        pattern::{ArrayPatternItem, ObjectPropertyPattern, Pattern, PropertyPattern, Rest},
    },
    value::{Value, ValueObjectMap},
};

use super::{
    env::{Environment, EMPTY_ENVIRONMENT},
    evaluation::Evaluation,
};

enum PatternFailPropagation<'s, 'v> {
    Shallow(PatternFailReason<'s, 'v>),
    Nested(PatternFail<'s, 'v>),
}

#[derive(Debug, Clone)]
pub struct PatternFail<'s, 'v> {
    pub reason: PatternFailReason<'s, 'v>,
    pub location: Option<Location>,
}

impl<'s, 'v> Pattern<'s> {
    fn cause_error(&self, reason: PatternFailReason<'s, 'v>) -> PatternFail<'s, 'v> {
        PatternFail {
            reason,
            location: self.location,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PatternFailReason<'s, 'v> {
    IdentifierConflict {
        identifier: Identifier<'s>,
        expected: Value<'s, 'v>,
        actual: Value<'s, 'v>,
    },
    ArrayLengthMismatch {
        expected: usize,
        actual: usize,
    },
    ArrayMinimumLengthMismatch {
        expected: usize,
        actual: usize,
    },
    TypeMismatch {
        expected: ValueType,
        actual: Value<'s, 'v>,
    },
    ObjectLengthMismatch {
        expected: usize,
        actual: usize,
    },
    ObjectKeyMismatch {
        expected: Cow<'s, str>,
        actual: ValueObjectMap<'s, 'v>,
    },
    EvalError(Box<EvalError<'s, 'v>>),
    LiteralMismatch,
    ExpressionMissmatch {
        expected: Value<'s, 'v>,
        actual: Value<'s, 'v>,
    },
}

#[derive(Clone, Debug)]
pub struct Matcher<'i, 's, 'v, 'e> {
    pub outer_env: &'e Environment<'i, 's, 'v>,
}

impl<'i: 's, 's, 'v: 's, 'e> Matcher<'i, 's, 'v, 'e> {
    pub fn match_pattern<'x>(
        &'x self,
        slf_env: Environment<'i, 's, 'v>,
        pattern: &'x Pattern<'s>,
        value: &Value<'s, 'v>,
    ) -> Result<Environment<'i, 's, 'v>, PatternFail<'s, 'v>> {
        match &pattern.body {
            PatternBody::Discard => Ok(slf_env),
            PatternBody::Capture(name, pat) => {
                self.match_pattern(slf_env, pat, value).and_then(|slf_env| {
                    self.match_identifier(slf_env, name, value)
                        .map_err(|e| pattern.cause_error(e))
                })
            }
            PatternBody::Identifier(name) => self
                .match_identifier(slf_env, name, value)
                .map_err(|e| pattern.cause_error(e)),
            PatternBody::TypedDiscard(t) => {
                if t == &value.get_type() {
                    Ok(slf_env)
                } else {
                    Err(pattern.cause_error(PatternFailReason::TypeMismatch {
                        expected: *t,
                        actual: value.clone(),
                    }))
                }
            }
            PatternBody::TypedIdentifier(name, t) => {
                if t != &value.get_type() {
                    return Err(pattern.cause_error(PatternFailReason::TypeMismatch {
                        expected: *t,
                        actual: value.clone(),
                    }));
                }
                self.match_identifier(slf_env, name, value)
                    .map_err(|e| pattern.cause_error(e))
            }
            PatternBody::Object(object_pattern, rest) => {
                let Value::Object(o) = value else {
                    return Err(pattern.cause_error(PatternFailReason::TypeMismatch {
                        expected: ValueType::Object,
                        actual: value.clone(),
                    }));
                };
                self.match_object(slf_env, object_pattern, rest, o)
                    .map_err(|propagation| match propagation {
                        PatternFailPropagation::Shallow(e) => pattern.cause_error(e),
                        PatternFailPropagation::Nested(e) => e,
                    })
            }
            PatternBody::Array(items, rest) => {
                let Value::Array(a) = value else {
                    return Err(pattern.cause_error(PatternFailReason::TypeMismatch {
                        expected: ValueType::Array,
                        actual: value.clone(),
                    }));
                };
                self.match_array(slf_env, items, rest, a)
                    .map_err(|propagation| match propagation {
                        PatternFailPropagation::Shallow(e) => pattern.cause_error(e),
                        PatternFailPropagation::Nested(e) => e,
                    })
            }
            PatternBody::Literal(l) => self
                .match_literal(slf_env, l, value)
                .map_err(|e| pattern.cause_error(e)),
            PatternBody::PinnedExpression(expr) => {
                let eval = Evaluation::new(self.outer_env);

                let exptected_value = match eval.eval_expr(expr) {
                    Err(e) => {
                        return Err(pattern.cause_error(PatternFailReason::EvalError(Box::new(e))))
                    }
                    Ok(expected) => expected,
                };

                if &exptected_value == value {
                    Ok(slf_env)
                } else {
                    Err(pattern.cause_error(PatternFailReason::ExpressionMissmatch {
                        expected: exptected_value,
                        actual: value.clone(),
                    }))
                }
            }
        }
    }

    fn match_identifier<'x>(
        &'x self,
        mut slf_env: Environment<'i, 's, 'v>,
        name: &'x Identifier<'x>,
        value: &Value<'s, 'v>,
    ) -> Result<Environment<'i, 's, 'v>, PatternFailReason<'s, 'v>> {
        match slf_env.bindings.entry(name.deep_clone()) {
            Entry::Occupied(entry) => {
                if value == entry.get() {
                    Ok(slf_env)
                } else {
                    Err(PatternFailReason::IdentifierConflict {
                        identifier: name.deep_clone(),
                        expected: entry.get().clone(),
                        actual: value.clone(),
                    })
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(value.clone());
                Ok(slf_env)
            }
        }
    }

    fn match_object<'x>(
        &'x self,
        mut slf_env: Environment<'i, 's, 'v>,
        props: &'x [ObjectPropertyPattern<'s>],
        rest: &Rest<'s>,
        value: &ValueObjectMap<'s, 'v>,
    ) -> Result<Environment<'i, 's, 'v>, PatternFailPropagation<'s, 'v>> {
        if let Rest::Exact = rest {
            if value.len() != props.len() {
                return Err(PatternFailPropagation::Shallow(
                    PatternFailReason::ObjectLengthMismatch {
                        expected: props.len(),
                        actual: value.len(),
                    },
                ));
            }
        }

        let mut keys = value.keys().collect::<BTreeSet<_>>();
        for prop in props {
            let (k, v) = match prop {
                ObjectPropertyPattern::Single(key) => (
                    key.name.clone(),
                    Pattern::new(PatternBody::Identifier(key.clone())),
                ),
                ObjectPropertyPattern::Match(PropertyPattern {
                    key: PropertyKey::Identifier(key),
                    value,
                }) => (key.name.clone(), value.clone()),
                ObjectPropertyPattern::Match(PropertyPattern {
                    key: PropertyKey::Expression(exp),
                    value,
                }) => {
                    let evaluation = Evaluation::new(self.outer_env);
                    match evaluation.eval_expr(exp) {
                        Ok(Value::String(k)) => (k.clone(), value.clone()),
                        Ok(v) => {
                            return Err(PatternFailPropagation::Shallow(
                                PatternFailReason::TypeMismatch {
                                    expected: ValueType::String,
                                    actual: v,
                                },
                            ))
                        }
                        Err(e) => {
                            return Err(PatternFailPropagation::Shallow(
                                PatternFailReason::EvalError(Box::new(e)),
                            ))
                        }
                    }
                }
            };

            if !keys.remove(&k) {
                return Err(PatternFailPropagation::Shallow(
                    PatternFailReason::ObjectKeyMismatch {
                        expected: k,
                        actual: value.clone(),
                    },
                ));
            }

            let Some(actual_value) = value.get(&k) else {
                return Err(PatternFailPropagation::Shallow(
                    PatternFailReason::ObjectKeyMismatch {
                        expected: k,
                        actual: value.clone(),
                    },
                ));
            };

            slf_env = self
                .match_pattern(slf_env, &v, actual_value.as_ref())
                .map_err(PatternFailPropagation::Nested)?
        }

        if let Rest::Collect(rest_pattern) = rest {
            let remaining: BTreeMap<Cow<str>, Cow<Value>> = keys
                .iter()
                .map(|&k| (k.clone(), value.get(k).unwrap().clone()))
                .collect();
            self.match_pattern(slf_env, rest_pattern, &Value::Object(remaining))
                .map_err(PatternFailPropagation::Nested)
        } else {
            Ok(slf_env)
        }
    }

    fn match_array<'x>(
        &'x self,
        mut slf_env: Environment<'i, 's, 'v>,
        items: &[ArrayPatternItem<'s>],
        rest: &Rest<'s>,
        value: &[Cow<'v, Value<'s, 'v>>],
    ) -> Result<Environment<'i, 's, 'v>, PatternFailPropagation<'s, 'v>> {
        if let Rest::Exact = rest {
            if value.len() != items.len() {
                return Err(PatternFailPropagation::Shallow(
                    PatternFailReason::ArrayLengthMismatch {
                        expected: items.len(),
                        actual: value.len(),
                    },
                ));
            }
        }

        if value.len() < items.len() {
            return Err(PatternFailPropagation::Shallow(
                PatternFailReason::ArrayMinimumLengthMismatch {
                    expected: items.len(),
                    actual: value.len(),
                },
            ));
        }

        for (ArrayPatternItem::Pattern(p), val) in std::iter::zip(items, value.iter()) {
            slf_env = self
                .match_pattern(slf_env, p, val.as_ref())
                .map_err(PatternFailPropagation::Nested)?
        }

        if let Rest::Collect(rest_pattern) = rest {
            self.match_pattern(
                slf_env,
                rest_pattern,
                &Value::Array(value.iter().skip(items.len()).cloned().collect()),
            )
            .map_err(PatternFailPropagation::Nested)
        } else {
            Ok(slf_env)
        }
    }

    fn match_literal(
        &self,
        slf_env: Environment<'i, 's, 'v>,
        literal: &Literal,
        value: &Value,
    ) -> Result<Environment<'i, 's, 'v>, PatternFailReason<'s, 'v>> {
        let matches = match (literal, value) {
            (Literal::Null, Value::Null) => true,
            (Literal::String(a), Value::String(b)) => a == b,
            (Literal::Number(n), Value::Integer(i)) => {
                str::parse::<i64>(n).map(|p| &p == i).unwrap_or(false)
            }
            (Literal::Boolean(a), Value::Boolean(b)) => a == b,
            (Literal::Type(a), Value::Type(b)) => a == b,
            _ => false,
        };

        if matches {
            Ok(slf_env)
        } else {
            Err(PatternFailReason::LiteralMismatch)
        }
    }

    pub fn new<'x: 'e>(env: &'x Environment<'i, 's, 'v>) -> Self {
        Self { outer_env: env }
    }
}

impl Default for Matcher<'_, '_, '_, 'static> {
    fn default() -> Self {
        Self {
            outer_env: &EMPTY_ENVIRONMENT,
        }
    }
}
