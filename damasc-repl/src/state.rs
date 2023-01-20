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

#[derive(Default)]
pub struct State<'i, 's, 'v> {
    environment: Environment<'i, 's, 'v>,
}

pub enum ReplOutput<'i, 's, 'v> {
    Ok,
    Write(String),
    Values(ValueBag<'s, 'v>),
    Bindings(Environment<'i, 's, 'v>),
    Exit,
}
pub enum ReplError {
    ParseError,
    EvalError,
    MatchError,
    TopologyError,
}



impl<'i, 's, 'v> State<'i, 's, 'v> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eval(&mut self, command: Command<'s, 's>) -> Result<ReplOutput<'i, 's, 'v>, ReplError> {
        match command {
            Command::Exit => Ok(ReplOutput::Exit),
            Command::Help => Ok(ReplOutput::Ok),
            Command::Cancel => Ok(ReplOutput::Ok),
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

                let transform_result = trans_iterator.flatten().flatten().collect::<Vec<_>>();

                Ok(ReplOutput::Values(ValueBag {
                    values: transform_result,
                }))
            }
            Command::Assign(assignments) => {
                let matcher = Matcher::new(&self.environment);
                match matcher.eval_assigment_set(assignments) {
                    Ok(new_bindings) => {
                        self.environment
                            .bindings
                            .append(&mut new_bindings.bindings.clone());
                        Ok(ReplOutput::Bindings(new_bindings))
                    }
                    Err(AssignmentError::EvalError) => Err(ReplError::EvalError),
                    Err(AssignmentError::MatchError) => Err(ReplError::MatchError),
                    Err(AssignmentError::TopologyError) => Err(ReplError::TopologyError),
                }
            }
            Command::Match(assignments) => {
                let matcher = Matcher::new(&self.environment);
                match matcher.eval_assigment_set(assignments) {
                    Ok(new_bindings) => Ok(ReplOutput::Bindings(new_bindings)),
                    Err(AssignmentError::EvalError) => Err(ReplError::EvalError),
                    Err(AssignmentError::MatchError) => Err(ReplError::MatchError),
                    Err(AssignmentError::TopologyError) => Err(ReplError::TopologyError),
                }
            }
            Command::Eval(assignments, expresions) => {
                let matcher = Matcher::new(&self.environment);
                let mut new_bindings = match matcher.eval_assigment_set(assignments) {
                    Ok(new_env) => new_env,
                    Err(AssignmentError::EvalError) => return Err(ReplError::EvalError),
                    Err(AssignmentError::MatchError) => return Err(ReplError::MatchError),
                    Err(AssignmentError::TopologyError) => return Err(ReplError::TopologyError),
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
                    Err(_err) => Err(ReplError::EvalError),
                }
            }
        }
    }
}
