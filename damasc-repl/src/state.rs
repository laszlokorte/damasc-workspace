use damasc_lang::{value::ValueBag, runtime::{env::Environment, evaluation::{self, Evaluation, EvalError}}};

use crate::{parser, command::Command};

pub struct State<'i, 's, 'v> {
    environment: Environment<'i, 's, 'v>,
}

pub enum ReplOutput<'i, 's,'v> {
    Ok,
    Write(String),
    Values(ValueBag<'s,'v>),
    Bindings(Environment<'i, 's, 'v>),
    Exit,
}
pub enum ReplError {
    ParseError,
    EvalError,
}

impl Default for State<'_, '_, '_> {
    fn default() -> Self {
        Self { environment: Default::default() }
    }
}

impl<'i, 's, 'v> State<'i, 's, 'v> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eval(&mut self, cmd_string: &str) -> Result<ReplOutput<'i, 's, 'v>, ReplError> {
        match parser::full_command(cmd_string) {
            Some(Command::Exit) => Ok(ReplOutput::Exit),
            Some(Command::Help) => Ok(ReplOutput::Ok),
            Some(Command::Cancel) => Ok(ReplOutput::Ok),
            Some(Command::Transform(t)) => {
                Ok(ReplOutput::Write(format!("{t:?}")))
            },
            Some(Command::Assign(a)) => {
                Ok(ReplOutput::Write(format!("{a:?}")))
            },
            Some(Command::Eval(a, e)) => {
                let evaluation = Evaluation::new(&self.environment);
                
                let values = e.expressions.into_iter()
                    .map(|e| evaluation.eval_expr(&e))
                    .collect::<Result<Vec<_>, EvalError>>();
                    match values {
                        Ok(v) => Ok(ReplOutput::Values(ValueBag{values: v})),
                        Err(err) => Err(ReplError::EvalError),
                    }
            },
            None => Err(ReplError::ParseError),
        }
    }
}