use crate::runtime::matching::PatternFail;
use crate::syntax::expression::IfElseExpression;
use crate::syntax::expression::MatchExpression;
use crate::syntax::location::Location;
use crate::value::LambdaBinding;
use crate::value::ValueArray;
use crate::value::ValueObjectMap;
use crate::value_type::ValueType;
use std::{borrow::Cow, collections::BTreeMap};

use itertools::Itertools;

use super::env::Environment;
use crate::runtime::matching::Matcher;
use crate::syntax::expression::ArrayComprehension;
use crate::syntax::expression::ComprehensionSource;
use crate::syntax::expression::LambdaAbstraction;
use crate::syntax::expression::LambdaApplication;
use crate::syntax::expression::ObjectComprehension;
use crate::value::Value;
use crate::{
    identifier::Identifier,
    literal::Literal,
    syntax::expression::{
        ArrayItem, BinaryExpression, BinaryOperator, CallExpression, Expression, ExpressionBody,
        LogicalExpression, LogicalOperator, MemberExpression, ObjectExpression, ObjectProperty,
        Property, PropertyKey, StringTemplate, UnaryExpression, UnaryOperator,
    },
};

#[derive(Debug, Clone)]
enum EvalErrorPropagation<'s, 'v> {
    Shallow(EvalErrorReason<'s, 'v>),
    Nested(EvalError<'s, 'v>),
}

impl<'s, 'v> EvalErrorPropagation<'s, 'v> {
    fn justify(self, expression: &Expression<'s>) -> EvalError<'s, 'v> {
        match self {
            Self::Shallow(e) => expression.cause_error(e),
            Self::Nested(e) => e,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvalError<'s, 'v> {
    pub reason: EvalErrorReason<'s, 'v>,
    pub location: Option<Location>,
}

impl<'s, 'v> Expression<'s> {
    fn cause_error(&self, reason: EvalErrorReason<'s, 'v>) -> EvalError<'s, 'v> {
        EvalError {
            reason,
            location: self.location,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalErrorReason<'s, 'v> {
    KindError(Value<'s, 'v>),
    TypeError(ValueType, Value<'s, 'v>),
    CollectionTypeError(Value<'s, 'v>),
    CastError(ValueType, Value<'s, 'v>),
    UnknownIdentifier(Identifier<'s>),
    InvalidNumber(String),
    MathDivisionByZero,
    KeyNotDefined(Value<'s, 'v>, Value<'s, 'v>),
    OutOfBound(usize, isize),
    IntegerOverflow,
    UnknownFunction(Identifier<'s>),
    PatternError(Box<PatternFail<'s, 'v>>),
    PatternExhaustionError(Value<'s, 'v>),
}

pub struct Evaluation<'e, 'i, 's, 'v> {
    env: &'e Environment<'i, 's, 'v>,
}

const EMPTY_ENV: &Environment = &Environment::new();

impl Default for Evaluation<'static, 'static, 'static, 'static> {
    fn default() -> Self {
        Self { env: EMPTY_ENV }
    }
}

impl<'e, 'i: 's, 's, 'v: 's> Evaluation<'e, 'i, 's, 'v> {
    pub fn new(env: &'e Environment<'i, 's, 'v>) -> Self {
        Self { env }
    }

    pub fn eval_expr<'x: 's, 'y>(
        &self,
        expression: &'y Expression<'x>,
    ) -> Result<Value<'s, 'v>, EvalError<'s, 'v>> {
        match &expression.body {
            ExpressionBody::Array(vec) => self.eval_array(vec).map_err(|e| e.justify(expression)),
            ExpressionBody::Binary(BinaryExpression {
                operator,
                left,
                right,
            }) => self.eval_expr(left).and_then(|l| {
                self.eval_expr(right).and_then(|r| {
                    self.eval_binary(operator, &l, &r)
                        .map_err(|e| e.justify(expression))
                })
            }),
            ExpressionBody::Identifier(id) => {
                self.eval_identifier(id).map_err(|e| e.justify(expression))
            }
            ExpressionBody::Literal(l) => self.eval_lit(l).map_err(|e| e.justify(expression)),
            ExpressionBody::Logical(LogicalExpression {
                operator,
                left,
                right,
            }) => self
                .eval_logic(operator, left, right)
                .map_err(|e| e.justify(expression)),
            ExpressionBody::Member(MemberExpression {
                object, property, ..
            }) => self.eval_expr(object).and_then(move |obj| {
                self.eval_expr(property).and_then(move |prop| {
                    self.eval_member(&obj, &prop)
                        .map_err(|e| e.justify(expression))
                })
            }),
            ExpressionBody::Object(props) => {
                self.eval_object(props).map_err(|e| e.justify(expression))
            }
            ExpressionBody::Unary(UnaryExpression {
                operator, argument, ..
            }) => self.eval_expr(argument).and_then(|v| {
                self.eval_unary(operator, &v)
                    .map_err(|e| e.justify(expression))
            }),
            ExpressionBody::Call(CallExpression { function, argument }) => self
                .eval_call(function, &self.eval_expr(argument)?)
                .map_err(|e| e.justify(expression)),
            ExpressionBody::Template(template) => self
                .eval_template(template)
                .map_err(|e| e.justify(expression)),
            ExpressionBody::Abstraction(LambdaAbstraction { arguments, body }) => {
                let new_env = match LambdaBinding::try_from_env(
                    self.env,
                    body.get_identifiers(),
                    arguments.get_identifiers(),
                ) {
                    Ok(new_env) => new_env,
                    Err(missing_id) => {
                        return Err(expression.cause_error(EvalErrorReason::UnknownIdentifier(
                            missing_id.deep_clone(),
                        )))
                    }
                };

                Ok(Value::Lambda(new_env, arguments.clone(), *body.clone()))
            }
            ExpressionBody::Application(app) => self
                .eval_application(app)
                .map_err(|e| e.justify(expression)),
            ExpressionBody::ArrayComp(comp) => self
                .eval_array_comprehension(comp)
                .map_err(|e| e.justify(expression)),
            ExpressionBody::ObjectComp(comp) => self
                .eval_object_comprehension(comp)
                .map_err(|e| e.justify(expression)),
            ExpressionBody::Match(matching) => {
                self.eval_match(matching).map_err(|e| e.justify(expression))
            }
            ExpressionBody::Condition(if_else) => self
                .eval_condition(if_else)
                .map_err(|e| e.justify(expression)),
        }
    }

    fn eval_lit<'x>(
        &self,
        literal: &'x Literal<'x>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        match literal {
            Literal::Null => Ok(Value::Null),
            Literal::String(s) => Ok(Value::<'s, 'v>::String(Cow::Owned(s.to_string()))),
            Literal::Number(s) => str::parse::<i64>(s)
                .map(Value::Integer)
                .map(Ok)
                .unwrap_or(Err(EvalErrorPropagation::Shallow(
                    EvalErrorReason::InvalidNumber(s.to_string()),
                ))),
            Literal::Boolean(b) => Ok(Value::Boolean(*b)),
            Literal::Type(t) => Ok(Value::Type(*t)),
        }
    }

    fn eval_binary(
        &self,
        op: &BinaryOperator,
        left: &Value<'s, 'v>,
        right: &Value<'s, 'v>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        match op {
            BinaryOperator::StrictEqual => Ok(Value::Boolean(left == right)),
            BinaryOperator::StrictNotEqual => Ok(Value::Boolean(left != right)),
            BinaryOperator::LessThan => {
                let Value::Integer(l) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        left.clone(),
                    )));
                };
                let Value::Integer(r) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        right.clone(),
                    )));
                };
                Ok(Value::Boolean(l < r))
            }
            BinaryOperator::GreaterThan => {
                let Value::Integer(l) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        left.clone(),
                    )));
                };
                let Value::Integer(r) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        right.clone(),
                    )));
                };
                Ok(Value::Boolean(l > r))
            }
            BinaryOperator::LessThanEqual => {
                let Value::Integer(l) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        left.clone(),
                    )));
                };
                let Value::Integer(r) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        right.clone(),
                    )));
                };
                Ok(Value::Boolean(l <= r))
            }
            BinaryOperator::GreaterThanEqual => {
                let Value::Integer(l) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        left.clone(),
                    )));
                };
                let Value::Integer(r) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        right.clone(),
                    )));
                };
                Ok(Value::Boolean(l >= r))
            }
            BinaryOperator::Plus => {
                let Value::Integer(l) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        left.clone(),
                    )));
                };
                let Value::Integer(r) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        right.clone(),
                    )));
                };
                l.checked_add(*r).map(Value::Integer).map(Ok).unwrap_or(Err(
                    EvalErrorPropagation::Shallow(EvalErrorReason::IntegerOverflow),
                ))
            }
            BinaryOperator::Minus => {
                let Value::Integer(l) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        left.clone(),
                    )));
                };
                let Value::Integer(r) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        right.clone(),
                    )));
                };
                l.checked_sub(*r).map(Value::Integer).map(Ok).unwrap_or(Err(
                    EvalErrorPropagation::Shallow(EvalErrorReason::IntegerOverflow),
                ))
            }
            BinaryOperator::Times => {
                let Value::Integer(l) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        left.clone(),
                    )));
                };
                let Value::Integer(r) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        right.clone(),
                    )));
                };
                l.checked_mul(*r).map(Value::Integer).map(Ok).unwrap_or(Err(
                    EvalErrorPropagation::Shallow(EvalErrorReason::IntegerOverflow),
                ))
            }
            BinaryOperator::Over => {
                let Value::Integer(l) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        left.clone(),
                    )));
                };
                let Value::Integer(r) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        right.clone(),
                    )));
                };
                if *r == 0 {
                    return Err(EvalErrorPropagation::Shallow(
                        EvalErrorReason::MathDivisionByZero,
                    ));
                }
                l.checked_div(*r).map(Value::Integer).map(Ok).unwrap_or(Err(
                    EvalErrorPropagation::Shallow(EvalErrorReason::IntegerOverflow),
                ))
            }
            BinaryOperator::Mod => {
                let Value::Integer(l) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        left.clone(),
                    )));
                };
                let Value::Integer(r) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        right.clone(),
                    )));
                };
                l.checked_rem(*r).map(Value::Integer).map(Ok).unwrap_or(Err(
                    EvalErrorPropagation::Shallow(EvalErrorReason::IntegerOverflow),
                ))
            }
            BinaryOperator::In => {
                let Value::String(s) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::String,
                        left.clone(),
                    )));
                };
                let Value::Object(o) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Object,
                        right.clone(),
                    )));
                };
                Ok(Value::Boolean(o.contains_key(s)))
            }
            BinaryOperator::PowerOf => {
                let Value::Integer(l) = left else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        left.clone(),
                    )));
                };
                let Value::Integer(r) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        right.clone(),
                    )));
                };
                l.checked_pow(*r as u32)
                    .map(Value::Integer)
                    .map(Ok)
                    .unwrap_or(Err(EvalErrorPropagation::Shallow(
                        EvalErrorReason::IntegerOverflow,
                    )))
            }
            BinaryOperator::Is => {
                let Value::Type(specified_type) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::KindError(
                        right.clone(),
                    )));
                };
                let actual_type = left.get_type();

                Ok(Value::Boolean(actual_type == *specified_type))
            }
            BinaryOperator::Cast => {
                let Value::Type(specified_type) = right else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::KindError(
                        right.clone(),
                    )));
                };

                let Some(v) = left.convert(*specified_type) else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::CastError(
                        *specified_type,
                        left.clone(),
                    )));
                };

                Ok(v)
            }
        }
    }

    fn eval_unary(
        &self,
        op: &UnaryOperator,
        arg: &Value<'s, 'v>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        match op {
            UnaryOperator::Minus => {
                let Value::Integer(v) = arg else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        arg.clone(),
                    )));
                };
                Ok(Value::Integer(-v))
            }
            UnaryOperator::Plus => {
                let Value::Integer(v) = arg else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        arg.clone(),
                    )));
                };
                Ok(Value::Integer(*v))
            }
            UnaryOperator::Not => {
                let Value::Boolean(b) = arg else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Boolean,
                        arg.clone(),
                    )));
                };
                Ok(Value::Boolean(!b))
            }
        }
    }

    fn eval_object<'x: 's, 'y>(
        &self,
        props: &'y ObjectExpression<'x>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        let target = BTreeMap::new();

        self.eval_into_object(target, props).map(Value::Object)
    }

    fn eval_into_object<'x: 's, 'y>(
        &self,
        mut into: ValueObjectMap<'s, 'v>,
        props: &'y ObjectExpression<'x>,
    ) -> Result<ValueObjectMap<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        for prop in props {
            match prop {
                ObjectProperty::Single(id @ Identifier { name }) => {
                    let keyval = Cow::Owned(name.to_string());
                    let valval = self.eval_identifier(id)?;

                    into.insert(keyval, Cow::Owned(valval.to_owned()));
                }
                ObjectProperty::Property(Property {
                    key,
                    value: value_expr,
                }) => {
                    let keyval = match key {
                        PropertyKey::Identifier(Identifier { name }) => {
                            Cow::Owned(name.to_string())
                        }
                        PropertyKey::Expression(e) => {
                            let val = self.eval_expr(e).map_err(EvalErrorPropagation::Nested)?;
                            let Value::String(s) = val else {
                                return Err(EvalErrorPropagation::Shallow(
                                    EvalErrorReason::TypeError(ValueType::String, val.clone()),
                                ));
                            };
                            s
                        }
                    };
                    let valval = self
                        .eval_expr(value_expr)
                        .map_err(EvalErrorPropagation::Nested)?;
                    into.insert(keyval, Cow::Owned(valval.to_owned()));
                }
                ObjectProperty::Spread(expr) => {
                    let to_spread = self.eval_expr(expr).map_err(EvalErrorPropagation::Nested)?;
                    let Value::Object(map) = to_spread else {
                        return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                            ValueType::Object,
                            to_spread.clone(),
                        )));
                    };
                    for (k, v) in map {
                        into.insert(k, v);
                    }
                }
            }
        }

        Ok(into)
    }

    fn eval_array<'x: 's, 'y>(
        &self,
        vec: &'y [ArrayItem<'x>],
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        let result = vec![];

        self.eval_into_array(result, vec).map(Value::Array)
    }

    fn eval_into_array<'x: 's, 'y>(
        &self,
        mut target: Vec<Cow<'v, Value<'s, 'v>>>,
        vec: &'y [ArrayItem<'x>],
    ) -> Result<ValueArray<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        for item in vec {
            match item {
                ArrayItem::Single(exp) => {
                    let v = self.eval_expr(exp).map_err(EvalErrorPropagation::Nested)?;

                    target.push(Cow::Owned(v));
                }
                ArrayItem::Spread(exp) => {
                    let v = self.eval_expr(exp).map_err(EvalErrorPropagation::Nested)?;
                    let Value::Array(mut multiples) = v else {
                        return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                            ValueType::Array,
                            v.clone(),
                        )));
                    };

                    target.append(&mut multiples);
                }
            }
        }

        Ok(target)
    }

    fn eval_logic<'x: 's, 'y>(
        &self,
        operator: &LogicalOperator,
        left: &'y Expression<'x>,
        right: &'y Expression<'x>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        let left_value = self.eval_expr(left).map_err(EvalErrorPropagation::Nested)?;
        let Value::Boolean(left_bool) = left_value else {
            return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                ValueType::Boolean,
                left_value.clone(),
            )));
        };
        if operator.short_circuit_on(left_bool) {
            return Ok(Value::Boolean(left_bool));
        }
        let right_value = self
            .eval_expr(right)
            .map_err(EvalErrorPropagation::Nested)?;
        let Value::Boolean(right_bool) = right_value else {
            return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                ValueType::Boolean,
                right_value.clone(),
            )));
        };
        return Ok(Value::Boolean(right_bool));
    }

    fn eval_member<'x: 'v>(
        &self,
        obj: &Value<'s, 'x>,
        prop: &Value<'s, 'x>,
    ) -> Result<Value<'s, 'x>, EvalErrorPropagation<'s, 'x>> {
        match obj {
            Value::Object(o) => {
                let Value::String(p) = prop else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::String,
                        prop.clone(),
                    )));
                };

                let Some(val) = o.get(p).map(|v| v.clone().into_owned()) else {
                    return Err(EvalErrorPropagation::Shallow(
                        EvalErrorReason::KeyNotDefined(obj.clone(), prop.clone()),
                    ));
                };

                Ok(val)
            }
            Value::Array(a) => {
                let Value::Integer(i) = prop else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        prop.clone(),
                    )));
                };
                let index = if *i < 0 {
                    let sub = i.unsigned_abs() as usize;
                    let len = a.len();
                    if len < sub {
                        return Err(EvalErrorPropagation::Shallow(EvalErrorReason::OutOfBound(
                            len,
                            *i as isize,
                        )));
                        
                    }
                    len - sub
                } else {
                    *i as usize
                };

                let Some(val) = a.get(index).map(|v| v.clone().into_owned()) else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::OutOfBound(
                        a.len(),
                        index as isize,
                    )));
                };

                Ok(val)
            }
            Value::String(s) => {
                let Value::Integer(i) = prop else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Integer,
                        prop.clone(),
                    )));
                };
                let index = if *i < 0 {
                    s.len() - i.unsigned_abs() as usize
                } else {
                    *i as usize
                };

                let Some(val) = s.chars().nth(index).map(|v| v.clone().to_string()) else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::OutOfBound(
                        s.len(),
                        index as isize,
                    )));
                };

                Ok(Value::String(Cow::Owned(val)))
            }
            // TODO: be more specific
            _ => Err(EvalErrorPropagation::Shallow(
                EvalErrorReason::CollectionTypeError(obj.clone()),
            )),
        }
    }

    fn eval_identifier(
        &self,
        id: &Identifier<'s>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        let Some(val) = self.env.bindings.get(id) else {
            return Err(EvalErrorPropagation::Shallow(
                EvalErrorReason::UnknownIdentifier(id.clone()),
            ));
        };

        Ok(val.clone())
    }

    fn eval_call(
        &self,
        function: &Identifier<'s>,
        argument: &Value<'s, 'v>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        Ok(match function.name.as_ref() {
            "length" => Value::Integer(match argument {
                Value::String(s) => s.len() as i64,
                Value::Array(a) => a.len() as i64,
                Value::Object(o) => o.len() as i64,
                _ => {
                    return Err(EvalErrorPropagation::Shallow(
                        EvalErrorReason::CollectionTypeError(argument.clone()),
                    ))
                }
            }),
            "keys" => Value::Array(match argument {
                Value::Object(o) => o
                    .keys()
                    .map(|k| Cow::Owned(Value::String(Cow::Owned(k.to_string()))))
                    .collect(),
                _ => {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Object,
                        argument.clone(),
                    )))
                }
            }),
            "values" => Value::Array(match argument {
                Value::Object(o) => o.values().cloned().collect(),
                _ => {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Object,
                        argument.clone(),
                    )))
                }
            }),
            "env" => Value::Object(match argument {
                Value::Lambda(env, _, _) => env
                    .bindings
                    .iter()
                    .map(|(k, v)| (Cow::Owned(k.to_string()), Cow::Owned(v.to_owned())))
                    .collect(),
                _ => {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Lambda,
                        argument.clone(),
                    )))
                }
            }),
            "rebind" => {
                match argument {
                    Value::Array(arr) => {
                        let Some(x) = arr.first() else {
                            return Err(EvalErrorPropagation::Shallow(
                                EvalErrorReason::OutOfBound(arr.len(), 1),
                            ));
                        };

                        let Value::Lambda(env, pattern, expression) = x.clone().into_owned() else {
                            return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                                ValueType::Lambda,
                                x.clone().into_owned(),
                            )));
                        };

                        let Some(y) = arr.get(1) else {
                            return Err(EvalErrorPropagation::Shallow(
                                EvalErrorReason::OutOfBound(arr.len(), 2),
                            ));
                        };

                        let Value::Object(obj) = y.clone().into_owned() else {
                            return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                                ValueType::Object,
                                y.clone().into_owned(),
                            )));
                        };

                        let mut new_env = env.clone();
                        new_env.replace(obj.into_iter().map(|(k, v)| {
                            (
                                Identifier::new_owned(k.into_owned()),
                                v.clone().into_owned(),
                            )
                        }));

                        Value::Lambda(new_env, pattern.clone(), expression.clone())
                    }
                    _ => {
                        return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                            ValueType::Array,
                            argument.clone(),
                        )))
                    }
                }
            }
            "type" => Value::Type(argument.get_type()),
            _ => {
                return Err(EvalErrorPropagation::Shallow(
                    EvalErrorReason::UnknownFunction(function.clone()),
                ))
            }
        })
    }

    fn eval_template<'x: 's, 'y>(
        &self,
        template: &'y StringTemplate<'x>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        let joined = template
            .parts
            .iter()
            .flat_map(move |part| {
                let prefix = Ok(Cow::Owned(part.fixed_start.as_ref().into()));
                let end_val = self
                    .eval_expr(&part.dynamic_end)
                    .map_err(EvalErrorPropagation::Nested);

                match end_val.clone().map(|v| v.convert(ValueType::String)) {
                    Ok(Some(Value::String(end))) => [prefix, Ok(end)],
                    Ok(_) => [
                        prefix,
                        end_val.and_then(|v| {
                            Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                                ValueType::String,
                                v.clone(),
                            )))
                        }),
                    ],
                    Err(e) => [prefix, Err(e)],
                }
            })
            .chain(Some(Ok(Cow::Owned(template.suffix.as_ref().into()))))
            .collect::<Result<Vec<Cow<'s, str>>, _>>()?;

        return Ok(Value::String(Cow::Owned(joined.join(""))));
    }

    fn eval_application<'x: 's>(
        &self,
        app: &LambdaApplication<'x>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        let lambda = self
            .eval_expr(&app.lambda)
            .map_err(EvalErrorPropagation::Nested)?;
        let param = self
            .eval_expr(&app.parameter)
            .map_err(EvalErrorPropagation::Nested)?;

        let Value::Lambda(bindings, pattern, lambda_body) = lambda else {
            return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                ValueType::Lambda,
                lambda.clone(),
            )));
        };

        let local_env = bindings.into();
        let matcher = Matcher::new(&local_env);

        let new_env = match matcher.match_pattern(Environment::new(), &pattern, &param) {
            Err(e) => {
                return Err(EvalErrorPropagation::Shallow(
                    EvalErrorReason::PatternError(Box::new(e)),
                ));
            }
            Ok(new_env) => new_env,
        };

        let local_env = matcher.outer_env.combine_with_override(&new_env);
        let local_eval = Evaluation::new(&local_env);

        local_eval
            .eval_expr(&lambda_body)
            .map_err(EvalErrorPropagation::Nested)
    }

    fn eval_array_comprehension<'x: 's>(
        &self,
        comp: &ArrayComprehension<'x>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        let mut envs: Box<dyn Iterator<Item = Result<Environment, EvalErrorPropagation<'s, 'v>>>> =
            comp.sources.iter().fold(
                Box::new(Some(Ok(self.env.clone())).into_iter()),
                |current_envs, source| {
                    Box::new(
                        current_envs
                            .map(|e| Evaluation::new(&e?).eval_comprehension_source(source))
                            .flatten_ok(),
                    )
                },
            );

        envs.try_fold(vec![], |result, e| {
            let binding = e?;
            let eval = Evaluation::new(&binding);

            eval.eval_into_array(result, &comp.projection)
        })
        .map(Value::Array)
    }

    fn eval_comprehension_source<'x: 's>(
        &self,
        source: &ComprehensionSource<'x>,
    ) -> Result<impl Iterator<Item = Environment<'i, 's, 'v>>, EvalErrorPropagation<'s, 'v>> {
        let expression_value: Value<'_, '_> = self
            .eval_expr(&source.collection)
            .map_err(EvalErrorPropagation::Nested)?;
        let Value::Array(vals) = expression_value else {
            return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                ValueType::Array,
                expression_value.clone(),
            )));
        };

        let mut results = vec![];

        for val in vals {
            let matcher = Matcher::new(self.env);
            let new_env = match matcher.match_pattern(Environment::new(), &source.pattern, &val) {
                Err(err) => {
                    if source.strong_pattern {
                        return Err(EvalErrorPropagation::Shallow(
                            EvalErrorReason::PatternError(Box::new(err)),
                        ));
                    } else {
                        continue;
                    }
                }
                Ok(new_env) => new_env,
            };

            let local_env = matcher.outer_env.combine_with_override(&new_env);
            let local_eval = Evaluation::new(&local_env);

            if let Some(p) = &source.predicate {
                let pred_result = local_eval
                    .eval_expr(p)
                    .map_err(EvalErrorPropagation::Nested)?;

                let Value::Boolean(pred_result_bool) = pred_result else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Boolean,
                        pred_result.clone(),
                    )));
                };

                if !pred_result_bool {
                    continue;
                }
            }

            results.push(local_env)
        }

        Ok(results.into_iter())
    }

    fn eval_object_comprehension<'x: 's>(
        &self,
        comp: &ObjectComprehension<'x>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        let mut envs: Box<dyn Iterator<Item = Result<Environment, EvalErrorPropagation<'s, 'v>>>> =
            comp.sources.iter().fold(
                Box::new(Some(Ok(self.env.clone())).into_iter()),
                |current_envs, source| {
                    Box::new(
                        current_envs
                            .map(|e| Evaluation::new(&e?).eval_comprehension_source(source))
                            .flatten_ok(),
                    )
                },
            );

        envs.try_fold(BTreeMap::new(), |result, e| {
            let binding = e?;
            let eval = Evaluation::new(&binding);

            eval.eval_into_object(result, &comp.projection)
        })
        .map(Value::Object)
    }

    fn eval_match<'x: 's>(
        &self,
        match_expr: &MatchExpression<'x>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        let subject_value = self
            .eval_expr(&match_expr.subject)
            .map_err(EvalErrorPropagation::Nested)?;

        for case in &match_expr.cases {
            let matcher = Matcher::new(self.env);
            let new_env =
                match matcher.match_pattern(Environment::new(), &case.pattern, &subject_value) {
                    Err(_) => continue,
                    Ok(new_env) => new_env,
                };

            let local_env = matcher.outer_env.combine_with_override(&new_env);
            let local_eval = Evaluation::new(&local_env);

            if let Some(guard) = &case.guard {
                let guard_val = local_eval
                    .eval_expr(guard)
                    .map_err(EvalErrorPropagation::Nested)?;

                let Value::Boolean(guard_bool) = guard_val else {
                    return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                        ValueType::Boolean,
                        guard_val.clone(),
                    )));
                };

                if !guard_bool {
                    continue;
                }
            }

            return local_eval
                .eval_expr(&case.body)
                .map_err(EvalErrorPropagation::Nested);
        }

        Err(EvalErrorPropagation::Shallow(
            EvalErrorReason::PatternExhaustionError(subject_value.clone()),
        ))
    }

    fn eval_condition<'x: 's>(
        &self,
        if_else: &IfElseExpression<'x>,
    ) -> Result<Value<'s, 'v>, EvalErrorPropagation<'s, 'v>> {
        let condition_value = self
            .eval_expr(&if_else.condition)
            .map_err(EvalErrorPropagation::Nested)?;

        let Value::Boolean(condition_bool) = condition_value else {
            return Err(EvalErrorPropagation::Shallow(EvalErrorReason::TypeError(
                ValueType::Boolean,
                condition_value.clone(),
            )));
        };

        if condition_bool {
            self.eval_expr(&if_else.true_branch)
                .map_err(EvalErrorPropagation::Nested)
        } else if let Some(ref fb) = &if_else.false_branch {
            self.eval_expr(fb).map_err(EvalErrorPropagation::Nested)
        } else {
            return Ok(Value::Null);
        }
    }
}
