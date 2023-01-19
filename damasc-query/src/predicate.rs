use damasc_lang::syntax::{pattern::Pattern, expression::Expression};

#[derive(Clone, Debug)]
pub struct Predicate<'s> {
    pub pattern: Pattern<'s>,
    pub guard: Expression<'s>,
}

#[derive(Clone, Debug)]
pub struct MultiPredicate<'s> {
    pub patterns: Vec<Pattern<'s>>,
    pub guard: Expression<'s>,
}

#[derive(Clone, Debug)]
pub struct Projection<'s> {
    pub predicate: Predicate<'s>,
    pub projection: Expression<'s>,
}

#[derive(Clone, Debug)]
pub struct MultiProjection<'s> {
    pub predicate: MultiPredicate<'s>,
    pub projections: Vec<Expression<'s>>,
}