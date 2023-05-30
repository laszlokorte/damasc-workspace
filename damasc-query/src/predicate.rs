use damasc_lang::{
    runtime::{env::Environment, evaluation::Evaluation},
    syntax::expression::Expression,
    value::Value,
};

use crate::capture::{Capture, MultiCapture};

#[derive(Debug, Clone)]
pub enum PredicateError {
    PatternError,
    GuardError,
}

#[derive(Clone, Debug)]
pub struct Predicate<'s> {
    pub capture: Capture<'s>,
    pub guard: Expression<'s>,
}

impl<'s> Predicate<'s> {
    pub fn apply<'v: 's, 'i: 's>(
        &self,
        env: &Environment<'i, 's, 'v>,
        value: &'v Value<'s, 'v>,
    ) -> Result<bool, PredicateError> {
        let env = match self.capture.apply(env, value) {
            Ok(Some(env)) => env,
            Ok(None) => return Ok(false),
            Err(_e) => return Err(PredicateError::PatternError),
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.guard) {
            Ok(Value::Boolean(b)) => Ok(b),
            Ok(_) => Err(PredicateError::GuardError),
            Err(_) => Err(PredicateError::GuardError),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MultiPredicate<'s> {
    pub capture: MultiCapture<'s>,
    pub guard: Expression<'s>,
}

impl<'s> MultiPredicate<'s> {
    pub(crate) fn apply<'v: 'x + 's, 'i: 's, 'e, 'x>(
        &self,
        env: &Environment<'i, 's, 'v>,
        values: impl Iterator<Item = &'x Value<'s, 'v>>,
    ) -> Result<bool, PredicateError> {
        let env = match self.capture.apply(env, values) {
            Ok(Some(e)) => e,
            Ok(None) => return Ok(false),
            Err(_e) => return Err(PredicateError::PatternError),
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.guard) {
            Ok(Value::Boolean(b)) => Ok(b),
            Ok(_) => Err(PredicateError::GuardError),
            Err(_) => Err(PredicateError::GuardError),
        }
    }
}
