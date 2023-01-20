use damasc_lang::syntax::{assignment::AssignmentSet, expression::ExpressionSet};
use damasc_query::transformation::Transformation;

#[derive(Debug, Clone)]
pub enum Command<'s> {
    Help,
    Cancel,
    Exit,
    Transform(Transformation<'s>),
    Assign(AssignmentSet<'s,'s>),
    Eval(AssignmentSet<'s,'s>, ExpressionSet<'s>),
}