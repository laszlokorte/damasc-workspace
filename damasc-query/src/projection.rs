use damasc_lang::identifier::Identifier;
use damasc_lang::literal::Literal;
use damasc_lang::runtime::env::Environment;
use damasc_lang::runtime::evaluation::Evaluation;
use damasc_lang::syntax::expression::Expression;
use damasc_lang::syntax::expression::ExpressionBody;
use damasc_lang::syntax::expression::ExpressionSet;
use damasc_lang::syntax::pattern::Pattern;
use damasc_lang::syntax::pattern::PatternBody;

use damasc_lang::syntax::pattern::PatternSet;
use damasc_lang::value::Value;

use crate::capture::MultiCapture;
use crate::predicate::MultiPredicate;
use crate::predicate::Predicate;
use crate::predicate::PredicateError;

#[derive(Debug)]
pub enum ProjectionError {
    PredicateError(PredicateError),
    EvalError,
}

#[derive(Clone, Debug)]
pub struct Projection<'s> {
    pub predicate: Predicate<'s>,
    pub projection: Expression<'s>,
}

impl<'s> Projection<'s> {
    pub fn apply<'v: 's, 'i: 's>(
        &self,
        env: &Environment<'i, 's, 'v>,
        value: &'v Value<'s, 'v>,
    ) -> Result<Option<Value<'s, 'v>>, ProjectionError> {
        let env = match self.predicate.capture.apply(env, value) {
            Ok(Some(env)) => env,
            Ok(None) => return Ok(None),
            Err(_e) => {
                return Err(ProjectionError::PredicateError(
                    PredicateError::PatternError,
                ))
            }
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.predicate.guard) {
            Ok(Value::Boolean(b)) => {
                if b {
                    let Ok(result) = evaluation.eval_expr(&self.projection) else {
                        return Err(ProjectionError::EvalError);
                    };

                    Ok(Some(result))
                } else {
                    Ok(None)
                }
            }
            Ok(_) => Err(ProjectionError::PredicateError(PredicateError::GuardError)),
            Err(_) => Err(ProjectionError::PredicateError(PredicateError::GuardError)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MultiProjection<'s> {
    pub predicate: MultiPredicate<'s>,
    pub projections: ExpressionSet<'s>,
}

impl Default for MultiProjection<'_> {
    fn default() -> Self {
        Self {
            predicate: MultiPredicate {
                capture: MultiCapture {
                    patterns: PatternSet {
                        patterns: vec![Pattern::new(PatternBody::Identifier(Identifier::new(
                            "$$",
                        )))],
                    },
                },
                guard: Expression::new(ExpressionBody::Literal(Literal::Boolean(true))),
            },
            projections: ExpressionSet {
                expressions: vec![Expression::new(ExpressionBody::Identifier(
                    Identifier::new("$$"),
                ))],
            },
        }
    }
}

impl<'s> MultiProjection<'s> {
    pub fn apply<'v: 'x + 's, 'i: 's, 'e, 'x>(
        &self,
        env: &Environment<'i, 's, 'v>,
        values: impl Iterator<Item = &'x Value<'s, 'v>>,
    ) -> Result<Option<Vec<Value<'s, 'v>>>, ProjectionError> {
        let env = match self.predicate.capture.apply(env, values) {
            Ok(Some(e)) => e,
            Ok(None) => return Ok(None),
            Err(_e) => {
                return Err(ProjectionError::PredicateError(
                    PredicateError::PatternError,
                ))
            }
        };

        let evaluation = Evaluation::new(&env);

        match evaluation.eval_expr(&self.predicate.guard) {
            Ok(Value::Boolean(b)) => {
                if b {
                    self.projections
                        .expressions
                        .iter()
                        .map(|p| {
                            evaluation
                                .eval_expr(p)
                                .map_err(|_| ProjectionError::EvalError)
                        })
                        .collect::<Result<Vec<Value>, ProjectionError>>()
                        .map(Some)
                } else {
                    Ok(None)
                }
            }
            Ok(_) => Err(ProjectionError::PredicateError(PredicateError::GuardError)),
            Err(_) => Err(ProjectionError::PredicateError(PredicateError::GuardError)),
        }
    }
}
