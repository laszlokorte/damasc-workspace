use crate::runtime::evaluation::EvalError;
use crate::syntax::location::Location;
use crate::syntax::pattern::PatternBody;
use crate::value_type::ValueType;
use std::collections::HashSet;
use std::{
    borrow::Cow,
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
};

use crate::{
    identifier::Identifier,
    literal::Literal,
    syntax::{
        assignment::{Assignment, AssignmentSet},
        expression::PropertyKey,
        pattern::{ArrayPatternItem, ObjectPropertyPattern, Pattern, PropertyPattern, Rest},
    },
    topology::TopologyError,
    value::{Value, ValueObjectMap},
};

use super::{
    env::{Environment, EMPTY_ENVIRONMENT},
    evaluation::Evaluation,
};

#[derive(Debug, Clone)]
pub struct PatternFail<'s, 'v> {
    pub reason: PatternFailReason<'s, 'v>,
    pub location: Option<Location>,
}

impl<'s, 'v> PatternFail<'s, 'v> {
    fn new(reason: PatternFailReason<'s, 'v>) -> Self {
        Self {
            reason,
            location: None,
        }
    }

    fn new_with_location(reason: PatternFailReason<'s, 'v>, location: Option<Location>) -> Self {
        Self { reason, location }
    }
}

impl<'s, 'v> Pattern<'s> {
    fn cause_error(&self, reason: PatternFailReason<'s, 'v>) -> PatternFail<'s, 'v> {
        PatternFail::new_with_location(reason, self.location)
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
    pub local_env: Environment<'i, 's, 'v>,
}

impl<'i: 's, 's, 'v: 's, 'e> Matcher<'i, 's, 'v, 'e> {
    pub fn into_env(mut self) -> Environment<'i, 's, 'v> {
        let mut result = self.outer_env.clone();
        result.bindings.append(&mut self.local_env.bindings);
        result
    }

    pub fn match_pattern<'x>(
        &'x mut self,
        pattern: &'x Pattern<'s>,
        value: &Value<'s, 'v>,
    ) -> Result<(), PatternFail<'s, 'v>> {
        match &pattern.body {
            PatternBody::Discard => Ok(()),
            PatternBody::Capture(name, pat) => self
                .match_pattern(pat, value)
                .and_then(|_| self.match_identifier(name, value)),
            PatternBody::Identifier(name) => self.match_identifier(name, value),
            PatternBody::TypedDiscard(t) => {
                if t == &value.get_type() {
                    Ok(())
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
                self.match_identifier(name, value)
            }
            PatternBody::Object(object_pattern, rest) => {
                let Value::Object(o) = value else {
                    return Err(pattern.cause_error(PatternFailReason::TypeMismatch {
                        expected: ValueType::Object,
                        actual: value.clone(),
                    }));
                };
                self.match_object(object_pattern, rest, o)
            }
            PatternBody::Array(items, rest) => {
                let Value::Array(a) = value else {
                    return Err(pattern.cause_error(PatternFailReason::TypeMismatch {
                        expected: ValueType::Array,
                        actual: value.clone(),
                    }));
                };
                self.match_array(items, rest, a)
            }
            PatternBody::Literal(l) => self.match_literal(l, value),
            PatternBody::PinnedExpression(expr) => {
                let eval = Evaluation::new(self.outer_env);

                let exptected_value = match eval.eval_expr(expr) {
                    Err(e) => {
                        return Err(pattern.cause_error(PatternFailReason::EvalError(Box::new(e))))
                    }
                    Ok(expected) => expected,
                };

                if &exptected_value == value {
                    Ok(())
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
        &'x mut self,
        name: &'x Identifier<'x>,
        value: &Value<'s, 'v>,
    ) -> Result<(), PatternFail<'s, 'v>> {
        match self.local_env.bindings.entry(name.deep_clone()) {
            Entry::Occupied(entry) => {
                if value == entry.get() {
                    Ok(())
                } else {
                    Err(PatternFail::new(PatternFailReason::IdentifierConflict {
                        identifier: name.deep_clone(),
                        expected: entry.get().clone(),
                        actual: value.clone(),
                    }))
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(value.clone());
                Ok(())
            }
        }
    }

    fn match_object<'x>(
        &'x mut self,
        props: &'x [ObjectPropertyPattern<'s>],
        rest: &Rest<'s>,
        value: &ValueObjectMap<'s, 'v>,
    ) -> Result<(), PatternFail<'s, 'v>> {
        if let Rest::Exact = rest {
            if value.len() != props.len() {
                return Err(PatternFail::new(PatternFailReason::ObjectLengthMismatch {
                    expected: props.len(),
                    actual: value.len(),
                }));
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
                            return Err(PatternFail::new(PatternFailReason::TypeMismatch {
                                expected: ValueType::String,
                                actual: v,
                            }))
                        }
                        Err(e) => {
                            return Err(PatternFail::new(PatternFailReason::EvalError(Box::new(e))))
                        }
                    }
                }
            };

            if !keys.remove(&k) {
                return Err(PatternFail::new(PatternFailReason::ObjectKeyMismatch {
                    expected: k,
                    actual: value.clone(),
                }));
            }

            let Some(actual_value) = value.get(&k) else {
                return Err(PatternFail::new(PatternFailReason::ObjectKeyMismatch {
                    expected: k,
                    actual: value.clone(),
                }));
            };

            self.match_pattern(&v, actual_value.as_ref())?
        }

        if let Rest::Collect(rest_pattern) = rest {
            let remaining: BTreeMap<Cow<str>, Cow<Value>> = keys
                .iter()
                .map(|&k| (k.clone(), value.get(k).unwrap().clone()))
                .collect();
            self.match_pattern(rest_pattern, &Value::Object(remaining))
        } else {
            Ok(())
        }
    }

    fn match_array<'x>(
        &'x mut self,
        items: &[ArrayPatternItem<'s>],
        rest: &Rest<'s>,
        value: &[Cow<'v, Value<'s, 'v>>],
    ) -> Result<(), PatternFail<'s, 'v>> {
        if let Rest::Exact = rest {
            if value.len() != items.len() {
                return Err(PatternFail::new(PatternFailReason::ArrayLengthMismatch {
                    expected: items.len(),
                    actual: value.len(),
                }));
            }
        }

        if value.len() < items.len() {
            return Err(PatternFail::new(
                PatternFailReason::ArrayMinimumLengthMismatch {
                    expected: items.len(),
                    actual: value.len(),
                },
            ));
        }

        for (ArrayPatternItem::Pattern(p), val) in std::iter::zip(items, value.iter()) {
            self.match_pattern(p, val.as_ref())?
        }

        if let Rest::Collect(rest_pattern) = rest {
            self.match_pattern(
                rest_pattern,
                &Value::Array(value.iter().skip(items.len()).cloned().collect()),
            )
        } else {
            Ok(())
        }
    }

    pub fn clear(&mut self) {
        self.local_env.bindings.clear();
    }

    fn match_literal(&self, literal: &Literal, value: &Value) -> Result<(), PatternFail<'s, 'v>> {
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
            Ok(())
        } else {
            Err(PatternFail::new(PatternFailReason::LiteralMismatch))
        }
    }

    pub fn new<'x: 'e>(env: &'x Environment<'i, 's, 'v>) -> Self {
        Self {
            outer_env: env,
            local_env: Environment::new(),
        }
    }

    pub fn new_with_local<'x: 'e>(
        outer_env: &'x Environment<'i, 's, 'v>,
        local_env: Environment<'i, 's, 'v>,
    ) -> Self {
        Self {
            outer_env,
            local_env,
        }
    }

    pub fn eval_assigment_set<'a: 's, 'b: 's>(
        &self,
        assignments: AssignmentSet<'a, 'b>,
    ) -> Result<Environment<'i, 's, 'v>, AssignmentError<'s, 'v>> {
        match assignments.sort_topological() {
            Ok(sorted_set) => {
                let mut local_env = self.outer_env.clone();
                let mut collected_env = Environment::default();

                for Assignment {
                    pattern,
                    expression,
                } in sorted_set.assignments
                {
                    let mut matcher = Matcher::new_with_local(&local_env, collected_env.clone());
                    let evaluation = Evaluation::new(&local_env);

                    let value = match evaluation.eval_expr(&expression) {
                        Ok(value) => value,
                        Err(e) => return Err(AssignmentError::EvalError(e)),
                    };

                    match matcher.match_pattern(&pattern, &value) {
                        Ok(()) => {
                            collected_env
                                .bindings
                                .append(&mut matcher.local_env.bindings.clone());
                            local_env = matcher.into_env();
                        }
                        Err(e) => return Err(AssignmentError::MatchError(e)),
                    }
                }

                Ok(collected_env)
            }
            Err(TopologyError::Cycle(c)) => Err(AssignmentError::TopologyError(c)),
        }
    }
}

impl Default for Matcher<'_, '_, '_, 'static> {
    fn default() -> Self {
        Self {
            outer_env: &EMPTY_ENVIRONMENT,
            local_env: Environment::new(),
        }
    }
}

#[derive(Debug)]
pub enum AssignmentError<'s, 'v> {
    TopologyError(HashSet<Identifier<'s>>),
    EvalError(EvalError<'s, 'v>),
    MatchError(PatternFail<'s, 'v>),
}
