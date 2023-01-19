use damasc_lang::syntax::expression::ExpressionSet;

use crate::projection::{MultiProjection};

#[derive(Debug, Clone)]
pub struct Transformation<'s> {
    pub bag: ExpressionSet<'s>,
    pub projection: MultiProjection<'s>,
}
