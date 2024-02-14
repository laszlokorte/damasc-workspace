use damasc_lang::runtime::env::Environment;
use damasc_lang::value::Value;
use itertools::Permutations;

use crate::predicate::MultiPredicate;
use crate::predicate::Predicate;
use crate::predicate::PredicateError;
use crate::projection::MultiProjection;
use crate::projection::Projection;
use crate::projection::ProjectionError;

pub struct PredicateIterator<'i, 's, 'v, It: Iterator> {
    env: Environment<'i, 's, 'v>,
    predicate: Predicate<'s>,
    iter: It,
}

impl<'i, 's, 'v, It: Iterator + Clone> Clone for PredicateIterator<'i, 's, 'v, It> {
    fn clone(&self) -> Self {
        Self {
            env: self.env.clone(),
            predicate: self.predicate.clone(),
            iter: self.iter.clone(),
        }
    }
}

impl<'i, 's, 'v, It: Iterator> PredicateIterator<'i, 's, 'v, It> {
    pub fn new(env: Environment<'i, 's, 'v>, predicate: Predicate<'s>, iter: It) -> Self {
        Self {
            env,
            predicate,
            iter,
        }
    }
}

impl<'i: 's, 's, I: Iterator<Item = &'s Value<'s, 's>>> Iterator
    for PredicateIterator<'i, 's, 's, I>
{
    type Item = Result<&'s Value<'s, 's>, PredicateError>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next()?;

        match self.predicate.apply(&self.env, item) {
            Ok(true) => Some(Ok(item)),
            Ok(false) => self.next(),
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct MultiPredicateIterator<'i, 's, 'v, It: Iterator> {
    env: Environment<'i, 's, 'v>,
    predicate: MultiPredicate<'s>,
    iter: Permutations<It>,
}

impl<'i, 's, 'v, It: Iterator + Clone> Clone for MultiPredicateIterator<'i, 's, 'v, It>
where
    It::Item: Clone,
{
    fn clone(&self) -> Self {
        Self {
            env: self.env.clone(),
            predicate: self.predicate.clone(),
            iter: self.iter.clone(),
        }
    }
}

impl<'i, 's, 'v, It: Iterator> MultiPredicateIterator<'i, 's, 'v, It>
where
    It::Item: Clone,
{
    pub fn new(env: Environment<'i, 's, 'v>, predicate: MultiPredicate<'s>, iter: It) -> Self {
        use itertools::Itertools;

        Self {
            env,
            iter: iter.permutations(predicate.capture.patterns.patterns.len()),
            predicate,
        }
    }
}

impl<'i: 's, 's, I: Iterator<Item = Value<'s, 's>>> Iterator
    for MultiPredicateIterator<'i, 's, 's, I>
{
    type Item = Result<Vec<Value<'s, 's>>, PredicateError>;

    fn next(&mut self) -> Option<Self::Item> {
        let items = self.iter.next()?;

        match self.predicate.apply(&self.env, items.iter()) {
            Ok(true) => Some(Ok(items)),
            Ok(false) => self.next(),
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct ProjectionIterator<'i, 's, 'v, It: Iterator> {
    env: Environment<'i, 's, 'v>,
    projection: Projection<'s>,
    iter: It,
}

impl<'i, 's, 'v, It: Iterator + Clone> Clone for ProjectionIterator<'i, 's, 'v, It>
where
    It::Item: Clone,
{
    fn clone(&self) -> Self {
        Self {
            env: self.env.clone(),
            projection: self.projection.clone(),
            iter: self.iter.clone(),
        }
    }
}

impl<'i, 's, 'v, It: Iterator> ProjectionIterator<'i, 's, 'v, It> {
    pub fn new(env: Environment<'i, 's, 'v>, projection: Projection<'s>, iter: It) -> Self {
        Self {
            env,
            projection,
            iter,
        }
    }
}

impl<'i: 's, 's, I: Iterator<Item = &'s Value<'s, 's>>> Iterator
    for ProjectionIterator<'i, 's, 's, I>
{
    type Item = Result<Value<'s, 's>, ProjectionError>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next()?;

        match self.projection.apply(&self.env, item) {
            Ok(Some(v)) => Some(Ok(v)),
            Ok(None) => self.next(),
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct MultiProjectionIterator<'i, 's, 'v, It: Iterator> {
    env: Environment<'i, 's, 'v>,
    projection: MultiProjection<'s>,
    iter: Permutations<It>,
}

impl<'i, 's, 'v, It: Iterator + Clone> Clone for MultiProjectionIterator<'i, 's, 'v, It>
where
    It::Item: Clone,
{
    fn clone(&self) -> Self {
        Self {
            env: self.env.clone(),
            projection: self.projection.clone(),
            iter: self.iter.clone(),
        }
    }
}

impl<'i, 's, 'v, It: Iterator> MultiProjectionIterator<'i, 's, 'v, It> {
    pub fn new(env: Environment<'i, 's, 'v>, projection: MultiProjection<'s>, iter: It) -> Self
    where
        It::Item: Clone,
    {
        use itertools::Itertools;

        Self {
            env,
            iter: iter.permutations(projection.predicate.capture.patterns.patterns.len()),
            projection,
        }
    }
}

impl<'i: 's, 's, I: Iterator<Item = Value<'s, 's>>> Iterator
    for MultiProjectionIterator<'i, 's, 's, I>
{
    type Item = Result<Vec<Value<'s, 's>>, ProjectionError>;

    fn next(&mut self) -> Option<Self::Item> {
        let items = self.iter.next()?;

        match self.projection.apply(&self.env, &mut items.iter()) {
            Ok(Some(vs)) => Some(Ok(vs)),
            Ok(None) => self.next(),
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct IndexedPredicateIterator<'i, 's, 'v, It: Iterator> {
    env: Environment<'i, 's, 'v>,
    predicate: Predicate<'s>,
    iter: It,
}

impl<'i, 's, 'v, It: Iterator + Clone> Clone for IndexedPredicateIterator<'i, 's, 'v, It>
where
    It::Item: Clone,
{
    fn clone(&self) -> Self {
        Self {
            env: self.env.clone(),
            predicate: self.predicate.clone(),
            iter: self.iter.clone(),
        }
    }
}

impl<'i, 's, 'v, It: Iterator> IndexedPredicateIterator<'i, 's, 'v, It> {
    pub fn new(env: Environment<'i, 's, 'v>, predicate: Predicate<'s>, iter: It) -> Self {
        Self {
            env,
            predicate,
            iter,
        }
    }
}

impl<'i: 's, 's, I: Iterator<Item = (usize, &'s Value<'s, 's>)>> Iterator
    for IndexedPredicateIterator<'i, 's, 's, I>
{
    type Item = Result<(usize, &'s Value<'s, 's>), (usize, PredicateError)>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, item) = self.iter.next()?;

        match self.predicate.apply(&self.env, item) {
            Ok(true) => Some(Ok((index, item))),
            Ok(false) => self.next(),
            Err(e) => Some(Err((index, e))),
        }
    }
}
