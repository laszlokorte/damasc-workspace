use std::iter;

use damasc_lang::syntax::expression::Expression;
use damasc_lang::value::Value;
use damasc_lang::runtime::env::Environment;
use damasc_lang::runtime::matching::Matcher;
use damasc_lang::runtime::matching::PatternFail;
use damasc_lang::runtime::evaluation::Evaluation;

use crate::predicate::MultiPredicate;
use crate::predicate::Predicate;
use crate::predicate::PredicateError;


#[derive(Debug)]
pub enum ProjectionError {
    PredicateError(PredicateError),
    EvalError
}

#[derive(Clone, Debug)]
pub struct Projection<'s> {
    pub predicate: Predicate<'s>,
    pub projection: Expression<'s>,
}

impl<'s> Projection<'s> {
    pub fn apply<'v,'i,'e>(&self, env: &Environment<'i, 's, 'v>, value: &'v Value<'s, 'v>) -> Result<Option<Value<'s, 'v>>, ProjectionError> {
        let mut matcher = Matcher::new(&env);
        let env = match matcher.match_pattern(&self.predicate.pattern, &value) {
            Ok(()) => matcher.into_env(),
            Err(e) => {
               match e {
                PatternFail::EvalError => return Err(ProjectionError::PredicateError(PredicateError::PatternError)),
                _ => return Ok(None),
            } 
            },
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.predicate.guard) {
            Ok(Value::Boolean(b)) => {
                if b {
                    let Ok(result) = evaluation.eval_expr(&self.projection) else {
                        return Err(ProjectionError::EvalError);
                    };

                    return Ok(Some(result));
                } else {
                    Ok(None)
                }
            },
            Ok(_) => {
                Err(ProjectionError::PredicateError(PredicateError::GuardError))
            }
            Err(_) => {
                Err(ProjectionError::PredicateError(PredicateError::GuardError))
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct MultiProjection<'s> {
    pub predicate: MultiPredicate<'s>,
    pub projections: Vec<Expression<'s>>,
}

impl<'s> MultiProjection<'s> {
    pub fn apply<'v:'x,'i,'e,'x>(&self, env: &Environment<'i, 's, 'v>, values: impl Iterator<Item=&'x Value<'s, 'v>>) -> Result<Option<Vec<Value<'s, 'v>>>, ProjectionError> {
        let mut matcher = Matcher::new(&env);
        let zipped = iter::zip(self.predicate.patterns.iter(), values);
        let result = zipped.fold(Ok(()), |prev, (pat, val)| prev.and(matcher.match_pattern(&pat, &val)));
        let env = match result {
            Ok(()) => matcher.into_env(),
            Err(e) => {
               match e {
                PatternFail::EvalError => return Err(ProjectionError::PredicateError(PredicateError::PatternError)),
                _ => return Ok(None),
            } 
            },
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.predicate.guard) {
            Ok(Value::Boolean(b)) => {
                if b {
                    self.projections.iter().map(|p| {
                        evaluation.eval_expr(&p).map_err(|_| ProjectionError::EvalError)
                    }).collect::<Result<Vec<Value>, ProjectionError>>().map(Some)
                } else {
                    return Ok(None)
                }
            },
            Ok(_) => {
                Err(ProjectionError::PredicateError(PredicateError::GuardError))
            }
            Err(_) => {
                Err(ProjectionError::PredicateError(PredicateError::GuardError))
            },
        }
    }
}