use damasc_lang::syntax::{assignment::AssignmentSet, expression::ExpressionSet};
use damasc_query::transformation::Transformation;

#[derive(Debug, Clone)]
pub enum Command<'a,'b> {
    Help,
    Cancel,
    Exit,
    Transform(Transformation<'a,'b>),
    Assign(AssignmentSet<'a,'b>),
    Match(AssignmentSet<'a,'b>),
    Eval(AssignmentSet<'a,'b>, ExpressionSet<'a>),
}