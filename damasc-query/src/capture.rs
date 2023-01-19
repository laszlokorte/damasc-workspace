use std::iter;

use damasc_lang::{syntax::{pattern::{Pattern, PatternSet}}, value::Value, runtime::{matching::{Matcher, PatternFail}, env::Environment}};

#[derive(Debug)]
pub enum CaptureError {
    PatternError
}

#[derive(Clone, Debug)]
pub struct Capture<'s> {
    pub pattern: Pattern<'s>,
}

impl<'s> Capture<'s> {
    pub fn apply<'v,'i,'e>(&self, env: &Environment<'i, 's, 'v>, value: &'v Value<'s, 'v>) -> Result<Option<Environment<'i, 's, 'v>>, CaptureError> {
        let mut matcher = Matcher::new(env);

        match matcher.match_pattern(&self.pattern, &value) {
            Ok(()) => Ok(Some(matcher.into_env())),
            Err(e) => {
               match e {
                    PatternFail::EvalError => return Err(CaptureError::PatternError),
                    _ => return Ok(None),
                } 
            },
        }
    }
}


#[derive(Clone, Debug)]
pub struct  MultiCapture<'s> {
    pub patterns: PatternSet<'s>,
}

impl<'s>  MultiCapture<'s> {
    pub fn apply<'v:'x,'i,'e,'x>(&self, env: &Environment<'i, 's, 'v>, values: impl Iterator<Item = &'x Value<'s, 'v>>) -> Result<Option<Environment<'i, 's, 'v>>, CaptureError> {
        let mut matcher = Matcher::new(&env);
        let zipped = iter::zip(self.patterns.patterns.iter(), values);
        let result = zipped.fold(Ok(()), |prev, (pat, val)| prev.and(matcher.match_pattern(&pat, &val)));
        match result {
            Ok(()) => Ok(Some(matcher.into_env())),
            Err(e) => {
               match e {
                    PatternFail::EvalError => return Err(CaptureError::PatternError),
                    _ => return Ok(None),
                } 
            },
        }
    }
}


