use damasc_query::predicate::PredicateError;
use damasc_query::projection::ProjectionError;
use itertools::Itertools;
use std::collections::BTreeSet;

use damasc_lang::identifier::Identifier;
use damasc_lang::runtime::evaluation::Evaluation;
use damasc_lang::{
    runtime::{
        env::Environment,
        evaluation::EvalError,
        matching::{AssignmentError, Matcher},
    },
    value::ValueBag,
};
use damasc_query::iter::MultiProjectionIterator;

use crate::command::Command;
use crate::io::{ReplError, ReplOutput};

#[derive(Default)]
pub struct State<'i: 's, 's> {
    environment: Environment<'i, 's, 's>,
}

impl<'i, 's> State<'i, 's> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vars<'x>(&'x self) -> BTreeSet<&'x Identifier<'i>> {
        self.environment.bindings.keys().collect()
    }

    pub fn eval(&mut self, command: Command<'s, 's>) -> Result<ReplOutput<'i, 's>, ReplError<'s, 's>> {
        match command {
            Command::Exit => Ok(ReplOutput::Exit),
            Command::Help => Ok(ReplOutput::Ok),
            Command::Cancel => Ok(ReplOutput::Ok),
            Command::ShowEnv => Ok(ReplOutput::Bindings(self.environment.clone())),
            Command::ClearEnv => {
                self.environment.clear();
                Ok(ReplOutput::Ok)
            }
            Command::Transform(transformation) => {
                let evaluation = Evaluation::new(&self.environment);
                let iter = transformation
                    .bag
                    .expressions
                    .iter()
                    .filter_map(|e| evaluation.eval_expr(e).ok());
                let trans_iterator = MultiProjectionIterator::new(
                    self.environment.clone(),
                    transformation.projection,
                    iter,
                );

                let transform_result = trans_iterator.flatten_ok().collect::<Result<Vec<_>, _>>();

                Ok(ReplOutput::Values(ValueBag {
                    values: transform_result.map_err(|e| match e {
                        ProjectionError::PredicateError(PredicateError::PatternError) => {
                            ReplError::TransformError
                        }
                        ProjectionError::PredicateError(PredicateError::GuardError) => {
                            ReplError::TransformError
                        }
                        ProjectionError::EvalError => ReplError::TransformError,
                    })?,
                }))
            }
            Command::Assign(assignments, locals) => {
                let local_env = if let Some(loc) = locals {
                    let local_matcher = Matcher::new(&self.environment);
                    match local_matcher.eval_assigment_set(loc) {
                        Ok(mut new_bindings) => {
                            let mut local_env = self.environment.clone();
                            local_env.bindings.append(&mut new_bindings.bindings);
                            local_env
                        }
                        Err(AssignmentError::EvalError(e)) => return Err(ReplError::EvalError(e)),
                        Err(AssignmentError::MatchError(e)) => return Err(ReplError::MatchError(e)),
                        Err(AssignmentError::TopologyError(e)) => {
                            return Err(ReplError::TopologyError(e))
                        }
                    }
                } else {
                    self.environment.clone()
                };

                let matcher = Matcher::new(&local_env);
                match matcher.eval_assigment_set(assignments) {
                    Ok(new_bindings) => {
                        self.environment
                            .bindings
                            .append(&mut new_bindings.bindings.clone());
                        Ok(ReplOutput::Bindings(new_bindings))
                    }
                    Err(AssignmentError::EvalError(e)) => Err(ReplError::EvalError(e)),
                    Err(AssignmentError::MatchError(e)) => Err(ReplError::MatchError(e)),
                    Err(AssignmentError::TopologyError(e)) => Err(ReplError::TopologyError(e)),
                }
            }
            Command::Match(assignments) => {
                let matcher = Matcher::new(&self.environment);
                match matcher.eval_assigment_set(assignments) {
                    Ok(new_bindings) => Ok(ReplOutput::Bindings(new_bindings)),
                    Err(AssignmentError::EvalError(e)) => Err(ReplError::EvalError(e)),
                    Err(AssignmentError::MatchError(e)) => Err(ReplError::MatchError(e)),
                    Err(AssignmentError::TopologyError(e)) => Err(ReplError::TopologyError(e)),
                }
            }
            Command::Eval(assignments, expresions) => {
                let matcher = Matcher::new(&self.environment);
                let mut new_bindings = match matcher.eval_assigment_set(assignments) {
                    Ok(new_env) => new_env,
                    Err(AssignmentError::EvalError(e)) => { return Err(ReplError::EvalError(e))},
                    Err(AssignmentError::MatchError(e)) => { return Err(ReplError::MatchError(e))},
                    Err(AssignmentError::TopologyError(e)) => { return Err(ReplError::TopologyError(e))},
                };
                let mut local_env = self.environment.clone();
                local_env.bindings.append(&mut new_bindings.bindings);

                let evaluation = Evaluation::new(&local_env);

                let values = expresions
                    .expressions
                    .into_iter()
                    .map(|e| evaluation.eval_expr(&e))
                    .collect::<Result<Vec<_>, EvalError>>();
                match values {
                    Ok(v) => Ok(ReplOutput::Values(ValueBag { values: v })),
                    Err(e) => Err(ReplError::EvalError(e)),
                }
            }
        }
    }
}
