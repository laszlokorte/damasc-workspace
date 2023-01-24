use std::collections::HashMap;

use damasc_lang::{
    identifier::Identifier,
    syntax::{
        assignment::AssignmentSet,
        expression::{Expression, ExpressionSet},
    },
    value::ValueBag,
};
use damasc_query::predicate::MultiPredicate;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum JoinSource<'s, 'v> {
    Constant(ValueBag<'s, 'v>),
    Named(Identifier<'s>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum JoinSink<'s> {
    Print,
    Named(Identifier<'s>),
}

#[derive(Clone, Debug)]
pub struct Join<'s, 'v> {
    pub input: HashMap<JoinSource<'s, 'v>, MultiPredicate<'s>>,
    pub output: HashMap<JoinSink<'s>, ExpressionSet<'s>>,
    pub local_assignments: AssignmentSet<'s, 'v>,
    pub guard: Expression<'s>,
}
