use ariadne::Config;
use ariadne::ReportBuilder;
use ariadne::{Label, Report, ReportKind, Source};
use damasc_lang::runtime::evaluation::EvalErrorReason;
use damasc_lang::runtime::matching::PatternFailReason;
use damasc_repl::io::ReplError;
use std::io::Write;
use std::ops::Range;

pub(crate) fn print_error(input: &str, e: &ReplError, out_buffer: &mut Vec<u8>) -> bool {
    match e {
        ReplError::ParseError => write!(out_buffer, "Parse Error").is_ok(),
        ReplError::EvalError(eval_error) => {
            let Some(source_location) = eval_error.location else {
                return write!(out_buffer, "Evaluation Error at unknown source location.").is_ok();
            };

            let builder = Report::build(ReportKind::Error, "REPL", source_location.start);

            let builder = builder
                .with_code("Evaluation")
                .with_config(Config::default().with_color(false));

            let builder = builder.with_message(match &eval_error.reason {
                EvalErrorReason::KindError(actual) => {
                    format!("Expected a type, but found {}.", actual)
                }
                EvalErrorReason::TypeError(expected_type, value) => format!(
                    "Expected a value of type {} but found {} of type {}.",
                    expected_type,
                    value,
                    value.get_type()
                ),
                EvalErrorReason::CollectionTypeError(val) => format!(
                    "Value must be an Array, Object or String, but was {} of type {}.",
                    val,
                    val.get_type()
                ),
                EvalErrorReason::CastError(expected_type, val) => format!(
                    "Value {} of type {} can not be converted into {}..",
                    val,
                    val.get_type(),
                    expected_type
                ),
                EvalErrorReason::UnknownIdentifier(identifier) => {
                    format!("Unknown identifier {}.", identifier)
                }
                EvalErrorReason::InvalidNumber(lit) => {
                    format!("Literal {} is not a valid number.", lit)
                }
                EvalErrorReason::MathDivisionByZero => "Division By Zero".to_string(),
                EvalErrorReason::KeyNotDefined(key, val) => {
                    format!("Object {} has no key {}.", val, key)
                }
                EvalErrorReason::OutOfBound(expected_lengnth, actual_length) => format!(
                    "Tried to access index {} of value that has a length of {}.",
                    actual_length, expected_lengnth,
                ),
                EvalErrorReason::IntegerOverflow => "Integer overflow".to_string(),
                EvalErrorReason::UnknownFunction(fun) => {
                    format!("Function of name {} does not exist.", fun)
                }
                EvalErrorReason::PatternError(_e) => {
                    "A pattern failed to match during evaluation.".to_string()
                }
                EvalErrorReason::PatternExhaustionError(val) => {
                    format!("None of the provided cases was a match for value {}.", val)
                }
            });

            let builder = builder.with_label(
                Label::new(("REPL", source_location.start..(source_location.end)))
                    .with_message("This expression failed to evaluate."),
            );

            builder
                .finish()
                .write(("REPL", Source::from(input)), out_buffer)
                .is_ok()
        }
        ReplError::MatchError(pattern_fail) => {
            let Some(source_location) = pattern_fail.location else {
                return write!(out_buffer, "Match Failed").is_ok();
            };

            let builder = Report::build(ReportKind::Error, "REPL", source_location.start)
                .with_config(Config::default().with_color(false));

            let builder = builder.with_code("Matching");

            let builder = builder.with_message(match &pattern_fail.reason {
                PatternFailReason::IdentifierConflict { identifier, expected, actual } => format!("Identifier {} is already bound to {} but is now matched against {}.", identifier, expected, actual),
                PatternFailReason::ArrayLengthMismatch { expected, actual } => format!("Array is expected to be of length {}. But has actual length {}.", expected, actual),
                PatternFailReason::ArrayMinimumLengthMismatch { expected, actual } => format!("Array is expected to be at least of length {}. But has actual length {}.", expected, actual),
                PatternFailReason::TypeMismatch { expected, actual } => format!("Value was expected to be of type {} but the actual type is {}.", expected, actual),
                PatternFailReason::ObjectLengthMismatch { expected, actual } => format!("Object is expected to have {} different fields. But the actual number of fields is {}.", expected, actual),
                PatternFailReason::ObjectKeyMismatch { expected, actual } => format!("The object was expected to have a key {}, but has only keys: {}.", expected, actual.keys().map(|k| k.as_ref()).intersperse(", ").collect::<String>()),
                PatternFailReason::LiteralMismatch => "The value does not match the expected literal.".to_string(),
                PatternFailReason::ExpressionMissmatch { expected, actual } => format!("The value is expected to be {} but actually was {}.", expected, actual),
                PatternFailReason::EvalError(_eval_error) => "During the pattern matching an evaulation error occured.".to_string(),
            });

            let builder = builder.with_label(
                Label::new(("REPL", source_location.start..(source_location.end)))
                    .with_message("This pattern failed to match."),
            );

            builder
                .finish()
                .write(("REPL", Source::from(input)), out_buffer)
                .is_ok()
        }
        ReplError::TopologyError(cycle) => {
            let builder: ReportBuilder<(&str, Range<usize>)> =
                Report::build(ReportKind::Error, "REPL", 0)
                    .with_config(Config::default().with_color(false));

            let builder = builder.with_code("Topology");

            let builder = builder.with_message(format!(
                "These defintions cyclicly depend on each other: {}",
                cycle
                    .iter()
                    .map(|k| k.name.as_ref())
                    .intersperse(", ")
                    .collect::<String>()
            ));

            builder
                .finish()
                .write(("REPL", Source::from(input)), out_buffer)
                .is_ok()
        }
        ReplError::TransformError => write!(out_buffer, "Error During Transformation").is_ok(),
    }
}
