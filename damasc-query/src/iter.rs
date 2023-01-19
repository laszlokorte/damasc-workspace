use damasc_lang::value::Value;
use damasc_lang::runtime::matching::Matcher;
use damasc_lang::runtime::matching::PatternFail;
use damasc_lang::runtime::env::Environment;
use damasc_lang::runtime::evaluation::Evaluation;
use itertools::Permutations;

use crate::predicate::PredicateError;
use crate::projection::MultiProjection;
use crate::projection::Projection;
use crate::predicate::Predicate;
use crate::predicate::MultiPredicate;
use crate::projection::ProjectionError;

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

        
        match self.predicate.apply(&self.env, item) {
            Ok(true) => Some(Ok(&item)),
            Ok(false) => self.next(),
            Err(e) => Some(Err(e)),
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

        match self.predicate.apply(&self.env, items.iter()) {
            Ok(true) => Some(Ok(items)),
            Ok(false) => self.next(),
            Err(e) => Some(Err(e)),
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

        match self.projection.apply(&self.env, item) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => self.next(),
            Err(e) => Some(Err(e)),
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

        match self.projection.apply(&self.env, &mut items.iter()) {
            Ok(Some(vs)) => Some(Ok(vs)),
            Ok(None) => self.next(),
            Err(e) => Some(Err(e)),
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


