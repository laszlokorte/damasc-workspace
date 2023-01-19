#[derive(Debug)]
pub enum PredicateError {
    PatternError,
    GuardError
}

#[derive(Debug)]
pub enum ProjectionError {
    PredicateError(PredicateError),
    EvalError
}