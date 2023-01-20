use damasc_lang::syntax::expression::ExpressionSet;

use crate::projection::MultiProjection;

#[derive(Debug, Clone)]
pub struct Transformation<'a, 'b> {
    pub bag: ExpressionSet<'a>,
    pub projection: MultiProjection<'b>,
}
