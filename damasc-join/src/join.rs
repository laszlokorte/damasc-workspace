use std::collections::HashMap;

use damasc_lang::{syntax::expression::ExpressionSet, identifier::Identifier};
use damasc_query::predicate::MultiPredicate;

#[derive(Debug, Hash, PartialEq, Eq)]
enum JoinSource<'s> {
    Constant(ExpressionSet<'s>),
    Named(Identifier<'s>),
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum JoinSink<'s> {
    Print(&'s str),
    Named(Identifier<'s>),
}

#[derive(Debug)]
struct Join<'s> {
    input: HashMap<JoinSource<'s>, MultiPredicate<'s>>,
    output: HashMap<JoinSink<'s>, ExpressionSet<'s>>,
}
