use std::iter;

use damasc_lang::{value::Value, runtime::{matching::{Matcher, PatternFail}, env::Environment}};
use damasc_lang::runtime::evaluation::Evaluation;
use itertools::Permutations;

use crate::{predicate::{Predicate, MultiPredicate, Projection, MultiProjection}, error::{PredicateError, ProjectionError}};

pub struct PredicateIterator<'i, 's, 'v, It:Iterator>  {
    env: Environment<'i, 's, 'v>,
    predicate: Predicate<'s>,
    iter: It,
}

impl<'i, 's, 'v, It:Iterator+Clone> Clone for PredicateIterator<'i, 's, 'v, It> {
    fn clone(&self) -> Self {
        Self { env: self.env.clone(), predicate: self.predicate.clone(), iter: self.iter.clone() }
    }
}

impl<'i, 's, 'v, It:Iterator> PredicateIterator<'i, 's, 'v, It> {
    pub fn new(env: Environment<'i, 's, 'v>, predicate: Predicate<'s>, iter: It) -> Self {
        Self {
            env,
            predicate,
            iter,
        }
    }
}

impl<'i, 's:'v,'v,I:Iterator<Item = &'v Value<'s, 'v>>> Iterator for PredicateIterator<'i, 's, 'v,I> {
    type Item = Result<&'v Value<'s, 'v>, PredicateError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(item) = self.iter.next() else {
            return None;
        };

        let mut matcher = Matcher::new(&self.env);
        let env = match matcher.match_pattern(&self.predicate.pattern, &item) {
            Ok(()) => matcher.into_env(),
            Err(e) => {
               match e {
                PatternFail::EvalError => return Some(Err(PredicateError::PatternError)),
                _ => return self.next(),
            } 
            },
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.predicate.guard) {
            Ok(Value::Boolean(b)) => {
                if b {
                    Some(Ok(item))
                } else {
                    self.next()
                }
            },
            Ok(_) => {
                Some(Err(PredicateError::GuardError))
            }
            Err(_) => {
                Some(Err(PredicateError::GuardError))
            },
        }
    }
}


pub struct MultiPredicateIterator<'i, 's, 'v, It:Iterator>  {
    env: Environment<'i, 's, 'v>,
    predicate: MultiPredicate<'s>,
    iter: Permutations<It>,
}

impl<'i, 's, 'v, It:Iterator+Clone> Clone for MultiPredicateIterator<'i, 's, 'v, It> where It::Item:Clone {
    fn clone(&self) -> Self {
        Self { env: self.env.clone(), predicate: self.predicate.clone(), iter: self.iter.clone() }
    }
}

impl<'i, 's, 'v, It:Iterator> MultiPredicateIterator<'i, 's, 'v, It> where It::Item: Clone {
    pub fn new(env: Environment<'i, 's, 'v>, predicate: MultiPredicate<'s>, iter: It) -> Self {
        use itertools::Itertools;

        Self {
            env,
            iter: iter.permutations(predicate.patterns.len()),
            predicate,
        }
    }
}

impl<'i, 's:'v,'v,I:Iterator<Item = Value<'s, 'v>>> Iterator for MultiPredicateIterator<'i, 's, 'v,I> {
    type Item = Result<Vec<Value<'s, 'v>>, PredicateError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(items) = self.iter.next() else {
            return None;
        };

        let mut matcher = Matcher::new(&self.env);
        let zipped = iter::zip(self.predicate.patterns.iter(), items.iter());
        let result = zipped.fold(Ok(()), |prev, (pat, val)| prev.and(matcher.match_pattern(&pat, &val)));
        let env = match result {
            Ok(()) => matcher.into_env(),
            Err(e) => {
               match e {
                PatternFail::EvalError => return Some(Err(PredicateError::PatternError)),
                _ => return self.next(),
            } 
            },
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.predicate.guard) {
            Ok(Value::Boolean(b)) => {
                if b {
                    Some(Ok(items))
                } else {
                    self.next()
                }
            },
            Ok(_) => {
                Some(Err(PredicateError::GuardError))
            }
            Err(_) => {
                Some(Err(PredicateError::GuardError))
            },
        }
    }
}





pub struct ProjectionIterator<'i, 's, 'v, It:Iterator>  {
    env: Environment<'i, 's, 'v>,
    projection: Projection<'s>,
    iter: It,
}

impl<'i, 's, 'v, It:Iterator+Clone> Clone for ProjectionIterator<'i, 's, 'v, It> where It::Item:Clone {
    fn clone(&self) -> Self {
        Self { 
            env: self.env.clone(), 
            projection: self.projection.clone(), 
            iter: self.iter.clone() 
        }
    }
}

impl<'i, 's, 'v, It:Iterator> ProjectionIterator<'i, 's, 'v, It> {
    pub fn new(env: Environment<'i, 's, 'v>, projection: Projection<'s>, iter: It) -> Self {
        Self {
            env,
            projection,
            iter,
        }
    }
}

impl<'i, 's:'v,'v,I:Iterator<Item = &'v Value<'s, 'v>>> Iterator for ProjectionIterator<'i, 's, 'v,I> {
    type Item = Result<Value<'s, 'v>, ProjectionError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(item) = self.iter.next() else {
            return None;
        };

        let mut matcher = Matcher::new(&self.env);
        let env = match matcher.match_pattern(&self.projection.predicate.pattern, &item) {
            Ok(()) => matcher.into_env(),
            Err(e) => {
               match e {
                PatternFail::EvalError => return Some(Err(ProjectionError::PredicateError(PredicateError::PatternError))),
                _ => return self.next(),
            } 
            },
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.projection.predicate.guard) {
            Ok(Value::Boolean(b)) => {
                if b {
                    let Ok(result) = evaluation.eval_expr(&self.projection.projection) else {
                        return Some(Err(ProjectionError::EvalError));
                    };

                    return Some(Ok(result));
                } else {
                    self.next()
                }
            },
            Ok(_) => {
                Some(Err(ProjectionError::PredicateError(PredicateError::GuardError)))
            }
            Err(_) => {
                Some(Err(ProjectionError::PredicateError(PredicateError::GuardError)))
            },
        }
    }
}


pub struct MultiProjectionIterator<'i, 's, 'v, It:Iterator>  {
    env: Environment<'i, 's, 'v>,
    projection: MultiProjection<'s>,
    iter: It,
}


impl<'i, 's, 'v, It:Iterator+Clone> Clone for MultiProjectionIterator<'i, 's, 'v, It> where It::Item:Clone {
    fn clone(&self) -> Self {
        Self { 
            env: self.env.clone(), 
            projection: self.projection.clone(), 
            iter: self.iter.clone() 
        }
    }
}

impl<'i, 's, 'v, It:Iterator> MultiProjectionIterator<'i, 's, 'v, It> {
    pub fn new(env: Environment<'i, 's, 'v>, projection: MultiProjection<'s>, iter: It) -> Self {
        Self {
            env,
            projection,
            iter,
        }
    }
}

impl<'i, 's:'v,'v,I:Iterator<Item = Vec<Value<'s, 'v>>>> Iterator for MultiProjectionIterator<'i, 's, 'v,I> {
    type Item = Result<Vec<Value<'s, 'v>>, ProjectionError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(items) = self.iter.next() else {
            return None;
        };

        let mut matcher = Matcher::new(&self.env);
        let zipped = iter::zip(self.projection.predicate.patterns.iter(), items.iter());
        let result = zipped.fold(Ok(()), |prev, (pat, val)| prev.and(matcher.match_pattern(&pat, &val)));
        let env = match result {
            Ok(()) => matcher.into_env(),
            Err(e) => {
               match e {
                PatternFail::EvalError => return Some(Err(ProjectionError::PredicateError(PredicateError::PatternError))),
                _ => return self.next(),
            } 
            },
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.projection.predicate.guard) {
            Ok(Value::Boolean(b)) => {
                if b {
                    Some(self.projection.projections.iter().map(|p| {
                        evaluation.eval_expr(&p).map_err(|_| ProjectionError::EvalError)
                    }).collect())
                } else {
                    self.next()
                }
            },
            Ok(_) => {
                Some(Err(ProjectionError::PredicateError(PredicateError::GuardError)))
            }
            Err(_) => {
                Some(Err(ProjectionError::PredicateError(PredicateError::GuardError)))
            },
        }
    }
}




pub struct IndexedPredicateIterator<'i, 's, 'v, It:Iterator>  {
    env: Environment<'i, 's, 'v>,
    predicate: Predicate<'s>,
    iter: It,
}


impl<'i, 's, 'v, It:Iterator+Clone> Clone for IndexedPredicateIterator<'i, 's, 'v, It> where It::Item:Clone {
    fn clone(&self) -> Self {
        Self { 
            env: self.env.clone(), 
            predicate: self.predicate.clone(), 
            iter: self.iter.clone() 
        }
    }
}

impl<'i, 's, 'v, It:Iterator> IndexedPredicateIterator<'i, 's, 'v, It> {
    pub fn new(env: Environment<'i, 's, 'v>, predicate: Predicate<'s>, iter: It) -> Self {
        Self {
            env,
            predicate,
            iter,
        }
    }
}

impl<'i, 's:'v,'v,I:Iterator<Item = (usize, Value<'s, 'v>)>> Iterator for IndexedPredicateIterator<'i, 's, 'v,I> {
    type Item = Result<(usize, Value<'s, 'v>), (usize, PredicateError)>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some((index, item)) = self.iter.next() else {
            return None;
        };

        let mut matcher = Matcher::new(&self.env);
        let env = match matcher.match_pattern(&self.predicate.pattern, &item) {
            Ok(()) => matcher.into_env(),
            Err(e) => {
               match e {
                PatternFail::EvalError => return Some(Err((index, PredicateError::PatternError))),
                _ => return self.next(),
            } 
            },
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.predicate.guard) {
            Ok(Value::Boolean(b)) => {
                if b {
                    Some(Ok((index, item)))
                } else {
                    self.next()
                }
            },
            Ok(_) => {
                Some(Err((index, PredicateError::GuardError)))
            }
            Err(_) => {
                Some(Err((index, PredicateError::GuardError)))
            },
        }
    }
}


