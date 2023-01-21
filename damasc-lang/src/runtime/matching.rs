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

use super::{env::{Environment, EMPTY_ENVIRONMENT}, evaluation::Evaluation};

#[derive(Debug)]
pub enum PatternFail {
    IdentifierConflict,
    ArrayMissmatch,
    ArrayLengthMismatch,
    TypeMismatch,
    ObjectMissmatch,
    ObjectLengthMismatch,
    ObjectKeyMismatch,
    EvalError,
    LiteralMismatch,
}

#[derive(Clone, Debug)]
pub struct Matcher<'i, 's, 'v, 'e> {
    pub outer_env: &'e Environment<'i, 's, 'v>,
    pub local_env: Environment<'i, 's, 'v>,
}

impl<'i, 's, 'v, 'e> Matcher<'i, 's, 'v, 'e> {
    pub fn into_env(mut self) -> Environment<'i, 's, 'v> {
        let mut result = self.outer_env.clone();
        result.bindings.append(&mut self.local_env.bindings);
        result
    }

    pub fn match_pattern<'x>(
        &'x mut self,
        pattern: &'x Pattern<'s>,
        value: &Value<'s, 'v>,
    ) -> Result<(), PatternFail> {
        match &pattern {
            Pattern::Discard => Ok(()),
            Pattern::Capture(name, pat) => self
                .match_pattern(pat, value)
                .and_then(|_| self.match_identifier(name, value)),
            Pattern::Identifier(name) => self.match_identifier(name, value),
            Pattern::TypedDiscard(t) => {
                if t == &value.get_type() {
                    Ok(())
                } else {
                    Err(PatternFail::TypeMismatch)
                }
            }
            Pattern::TypedIdentifier(name, t) => {
                if t != &value.get_type() {
                    return Err(PatternFail::TypeMismatch);
                }
                self.match_identifier(name, value)
            }
            Pattern::Object(pattern, rest) => {
                let Value::Object(o) = value else {
                    return Err(PatternFail::ObjectMissmatch);
                };
                self.match_object(pattern, rest, o)
            }
            Pattern::Array(items, rest) => {
                let Value::Array(a) = value else {
                    return Err(PatternFail::ArrayMissmatch);
                };
                self.match_array(items, rest, a)
            }
            Pattern::Literal(l) => self.match_literal(l, value),
        }
    }

    fn match_identifier<'x>(
        &'x mut self,
        name: &'x Identifier<'x>,
        value: &Value<'s, 'v>,
    ) -> Result<(), PatternFail> {
        let id = Identifier {
            name: Cow::Owned(name.name.to_string()),
        };

        match self.local_env.bindings.entry(id) {
            Entry::Occupied(entry) => {
                if value == entry.get() {
                    Ok(())
                } else {
                    Err(PatternFail::IdentifierConflict)
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
        props: &[ObjectPropertyPattern<'s>],
        rest: &Rest<'s>,
        value: &ValueObjectMap<'s, 'v>,
    ) -> Result<(), PatternFail> {
        if let Rest::Exact = rest {
            if value.len() != props.len() {
                return Err(PatternFail::ObjectLengthMismatch);
            }
        }

        let mut keys = value.keys().collect::<BTreeSet<_>>();
        for prop in props {
            let (k, v) = match prop {
                ObjectPropertyPattern::Single(key) => {
                    (key.name.clone(), Pattern::Identifier(key.clone()))
                }
                ObjectPropertyPattern::Match(PropertyPattern {
                    key: PropertyKey::Identifier(key),
                    value,
                }) => (key.name.clone(), value.clone()),
                ObjectPropertyPattern::Match(PropertyPattern {
                    key: PropertyKey::Expression(exp),
                    value,
                }) => {
                    let evaluation = Evaluation::new(self.outer_env);
                    let Ok(Value::String(k)) = evaluation.eval_expr(exp) else {
                        return Err(PatternFail::EvalError);
                    };
                    (k.clone(), value.clone())
                }
            };

            if !keys.remove(&k) {
                return Err(PatternFail::ObjectKeyMismatch);
            }

            let Some(actual_value) = value.get(&k) else {
                return Err(PatternFail::ObjectKeyMismatch);
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
        value: &Vec<Cow<'v, Value<'s, 'v>>>,
    ) -> Result<(), PatternFail> {
        if let Rest::Exact = rest {
            if value.len() != items.len() {
                return Err(PatternFail::ArrayLengthMismatch);
            }
        }

        if value.len() < items.len() {
            return Err(PatternFail::ArrayLengthMismatch);
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

    fn match_literal(&self, literal: &Literal, value: &Value) -> Result<(), PatternFail> {
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
            Err(PatternFail::LiteralMismatch)
        }
    }
    pub fn new<'x: 'e>(env: &'x Environment<'i, 's, 'v>) -> Self {
        Self {
            outer_env: env,
            local_env: Environment::new(),
        }
    }

    pub fn eval_assigment_set<'a: 's, 'b>(
        &self,
        assignments: AssignmentSet<'a, 'b>,
    ) -> Result<Environment<'i, 's, 'v>, AssignmentError> {
        match assignments.sort_topological() {
            Ok(sorted_set) => {
                let mut local_env = self.outer_env.clone();
                local_env
                    .bindings
                    .append(&mut self.local_env.bindings.clone());
                let mut collected_env = Environment::default();

                for Assignment {
                    pattern,
                    expression,
                } in sorted_set.assignments
                {
                    let mut matcher = Matcher::new(&local_env);
                    let evaluation = Evaluation::new(&local_env);
                    let Ok(value) = evaluation.eval_expr(&expression) else {
                        return Err(AssignmentError::EvalError);
                    };
                    match matcher.match_pattern(&pattern, &value) {
                        Ok(()) => {
                            collected_env
                                .bindings
                                .append(&mut matcher.local_env.bindings.clone());
                            local_env = matcher.into_env();
                        }
                        Err(_) => return Err(AssignmentError::MatchError),
                    }
                }

                Ok(collected_env)
            }
            Err(TopologyError::Cycle(_c)) => Err(AssignmentError::TopologyError),
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
pub enum AssignmentError {
    TopologyError,
    EvalError,
    MatchError,
}
