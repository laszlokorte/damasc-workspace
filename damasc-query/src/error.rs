pub(crate) enum PredicateError {
    PatternError,
    GuardError
}

pub(crate) enum ProjectionError {
    PredicateError(PredicateError),
    EvalError
}