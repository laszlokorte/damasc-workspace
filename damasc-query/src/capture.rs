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
        let mut matcher = Matcher::new(env);

        match matcher.match_pattern(&self.pattern, value) {
            Ok(()) => Ok(Some(matcher.into_env())),
            Err(e) => match e {
                PatternFail::EvalError => Err(CaptureError::PatternError),
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
        let mut matcher = Matcher::new(env);
        let zipped = iter::zip(self.patterns.patterns.iter(), values);
        let result = zipped.fold(Ok(()), |prev, (pat, val)| {
            prev.and(matcher.match_pattern(pat, val))
        });
        match result {
            Ok(()) => Ok(Some(matcher.into_env())),
            Err(e) => match e {
                PatternFail::EvalError => Err(CaptureError::PatternError),
                _ => Ok(None),
            },
        }
    }
}
