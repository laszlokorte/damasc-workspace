use std::iter;

use damasc_lang::{syntax::{pattern::Pattern, expression::Expression}, value::Value, runtime::{matching::{Matcher, PatternFail}, evaluation::Evaluation, env::Environment}};

#[derive(Debug)]
pub enum PredicateError {
    PatternError,
    GuardError
}

#[derive(Clone, Debug)]
pub struct Predicate<'s> {
    pub pattern: Pattern<'s>,
    pub guard: Expression<'s>,
}

impl<'s> Predicate<'s> {
    pub fn apply<'v,'i,'e>(&self, env: &Environment<'i, 's, 'v>, value: &'v Value<'s, 'v>) -> Result<bool, PredicateError> {
        let mut matcher = Matcher::new(env);

        let env = match matcher.match_pattern(&self.pattern, &value) {
            Ok(()) => matcher.into_env(),
            Err(e) => {
               match e {
                PatternFail::EvalError => return Err(PredicateError::PatternError),
                _ => return Ok(false),
            } 
            },
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.guard) {
            Ok(Value::Boolean(b)) => {
                Ok(b)
            },
            Ok(_) => {
                Err(PredicateError::GuardError)
            }
            Err(_) => {
                Err(PredicateError::GuardError)
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct MultiPredicate<'s> {
    pub patterns: Vec<Pattern<'s>>,
    pub guard: Expression<'s>,
}

impl<'s> MultiPredicate<'s> {
    pub(crate) fn apply<'v:'x,'i,'e,'x>(&self, env: &Environment<'i, 's, 'v>, values: impl Iterator<Item = &'x Value<'s, 'v>>) -> Result<bool, PredicateError> {
        let mut matcher = Matcher::new(&env);
        let zipped = iter::zip(self.patterns.iter(), values);
        let result = zipped.fold(Ok(()), 
            |prev, (pat, val)| prev.and(matcher.match_pattern(&pat, &val)));
        let env = match result {
            Ok(()) => matcher.into_env(),
            Err(e) => {
               match e {
                PatternFail::EvalError => return Err(PredicateError::PatternError),
                _ => return Ok(false),
            } 
            },
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.guard) {
            Ok(Value::Boolean(b)) => {
                Ok(b)
            },
            Ok(_) => {
                Err(PredicateError::GuardError)
            }
            Err(_) => {
                Err(PredicateError::GuardError)
            },
        }
    }


}
