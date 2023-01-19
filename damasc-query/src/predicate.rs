use damasc_lang::syntax::{pattern::Pattern, expression::Expression};

#[derive(Clone, Debug)]
pub(crate) struct Predicate<'s> {
    pub(crate) pattern: Pattern<'s>,
    pub(crate) guard: Expression<'s>,
}

#[derive(Clone, Debug)]
pub(crate) struct MultiPredicate<'s> {
    pub(crate) patterns: Vec<Pattern<'s>>,
    pub(crate) guard: Expression<'s>,
}

#[derive(Clone, Debug)]
pub(crate) struct Projection<'s> {
    pub(crate) predicate: Predicate<'s>,
    pub(crate) projection: Expression<'s>,
}

#[derive(Clone, Debug)]
pub(crate) struct MultiProjection<'s> {
    pub(crate) predicate: MultiPredicate<'s>,
    pub(crate) projections: Vec<Expression<'s>>,
}