use damasc_lang::{runtime::env::Environment, value::ValueBag};

#[derive(Debug)]
pub enum ReplOutput<'i, 's> {
    Ok,
    Write(String),
    Values(ValueBag<'s, 's>),
    Bindings(Environment<'i, 's, 's>),
    Exit,
}

impl std::fmt::Display for ReplOutput<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplOutput::Ok => writeln!(f, "OK."),
            ReplOutput::Write(msg) => writeln!(f, "{msg}"),
            ReplOutput::Values(vals) => writeln!(f, "{vals}"),
            ReplOutput::Bindings(env) => writeln!(f, "{env}"),
            ReplOutput::Exit => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum ReplError {
    ParseError,
    EvalError,
    MatchError,
    TopologyError,
}
