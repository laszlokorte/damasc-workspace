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
        let mut zipped = iter::zip(self.patterns.patterns.iter(), values);
        let result = zipped.try_fold(env.clone(), |e, (pat, val)| {
            let mut m = Matcher::new_with_local(&e, e.clone());
            m.match_pattern(pat, val)?;
            Ok(m.into_env())
        });

        match result {
            Ok(final_env) => Ok(Some(final_env)),
            Err(e) => match e {
                PatternFail::EvalError => Err(CaptureError::PatternError),
                _ => Ok(None),
            },
        }
    }
}
