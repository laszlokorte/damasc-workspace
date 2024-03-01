use damasc_lang::runtime::matching::PatternFailReason;
use std::iter;

use damasc_lang::{
    runtime::{
        env::Environment,
        matching::{Matcher, PatternFail},
    },
    syntax::pattern::{Pattern, PatternSet},
    value::Value,
};

#[derive(Debug)]
pub enum CaptureError {
    PatternError,
    EvalError,
}

#[derive(Clone, Debug)]
pub struct Capture<'s> {
    pub pattern: Pattern<'s>,
}

impl<'s> Capture<'s> {
    pub fn apply<'v: 's, 'i: 's>(
        &self,
        env: &Environment<'i, 's, 'v>,
        value: &'v Value<'s, 'v>,
    ) -> Result<Option<Environment<'i, 's, 'v>>, CaptureError> {
        let matcher = Matcher::new(env);

        match matcher.match_pattern(Environment::new(), &self.pattern, value) {
            Ok(new_env) => Ok(Some(matcher.outer_env.combine_with_override(&new_env))),
            Err(e) => match e.reason {
                PatternFailReason::EvalError(_e) => Err(CaptureError::EvalError),
                _ => Ok(None),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct MultiCapture<'s> {
    pub patterns: PatternSet<'s>,
}

impl<'s> MultiCapture<'s> {
    pub fn apply<'v: 'x + 's, 'i: 's, 'e, 'x>(
        &self,
        env: &Environment<'i, 's, 'v>,
        values: impl Iterator<Item = &'x Value<'s, 'v>>,
    ) -> Result<Option<Environment<'i, 's, 'v>>, CaptureError> {
        let mut zipped = iter::zip(self.patterns.patterns.iter(), values);
        let result: Result<Environment, PatternFail> =
            zipped.try_fold(env.clone(), |e, (pat, val)| {
                let m = Matcher::new(&e);
                let new_env = m.match_pattern(e.clone(), pat, val)?;
                Ok(m.outer_env.combine_with_override(&new_env))
            });

        match result {
            Ok(final_env) => Ok(Some(final_env)),
            Err(e) => match &e.reason {
                PatternFailReason::EvalError(_e) => Err(CaptureError::EvalError),
                _ => Ok(None),
            },
        }
    }
}
