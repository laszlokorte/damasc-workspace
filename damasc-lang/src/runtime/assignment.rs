use crate::identifier::Identifier;
use crate::runtime::env::Environment;
use crate::runtime::evaluation::EvalError;
use crate::runtime::evaluation::Evaluation;
use crate::runtime::matching::Matcher;
use crate::runtime::matching::PatternFail;
use crate::syntax::assignment::Assignment;
use crate::syntax::assignment::AssignmentSet;
use crate::topology::TopologyError;

use std::collections::HashSet;

#[derive(Debug)]
pub enum AssignmentError<'s, 'v> {
    TopologyError(HashSet<Identifier<'s>>),
    EvalError(EvalError<'s, 'v>),
    MatchError(PatternFail<'s, 'v>),
}

#[derive(Debug)]
pub struct AssignmentEvaluation<'i, 's, 'v, 'e> {
    env: &'e Environment<'i, 's, 'v>,
}

impl<'i: 's, 's, 'v: 's, 'e> AssignmentEvaluation<'i, 's, 'v, 'e> {
    pub fn new(env: &'e Environment<'i, 's, 'v>) -> Self {
        Self { env }
    }

    pub fn eval_assigment_set<'a: 's, 'b: 's>(
        &self,
        assignments: AssignmentSet<'a, 'b>,
    ) -> Result<Environment<'i, 's, 'v>, AssignmentError<'s, 'v>> {
        match assignments.sort_topological() {
            Ok(sorted_set) => {
                let mut local_env = self.env.clone();
                let mut collected_env = Environment::default();

                for Assignment {
                    pattern,
                    expression,
                } in sorted_set.assignments
                {
                    let matcher = Matcher::new(&local_env);
                    let evaluation = Evaluation::new(&local_env);

                    let value = match evaluation.eval_expr(&expression) {
                        Ok(value) => value,
                        Err(e) => return Err(AssignmentError::EvalError(e)),
                    };

                    match matcher.match_pattern(collected_env, &pattern, &value) {
                        Ok(new_env) => {
                            local_env.bindings.append(&mut new_env.bindings.clone());
                            collected_env = new_env;
                        }
                        Err(e) => return Err(AssignmentError::MatchError(e)),
                    }
                }

                Ok(collected_env)
            }
            Err(TopologyError::Cycle(c)) => Err(AssignmentError::TopologyError(c)),
        }
    }
}
