use damasc_lang::runtime::evaluation::EvalErrorReason;
use damasc_repl::io::ReplError;


use damasc_repl::{io::ReplOutput, parser, state::State};
use rustyline::{error::ReadlineError, Editor};

const HISTORY_FILE: &str = "history.txt";

fn main() -> rustyline::Result<()> {
    let mut repl = State::default();
    let mut rl = Editor::<()>::new()?;

    if rl.load_history(HISTORY_FILE).is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let cmd = match parser::command_all_consuming(&line) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        eprintln!("{e:?}");
                        continue;
                    }
                };

                match repl.eval(cmd) {
                    Ok(ReplOutput::Ok) => println!("Ok"),
                    Ok(ReplOutput::Exit) => break,
                    Ok(ReplOutput::Values(v)) => println!("{v}"),
                    Ok(ReplOutput::Bindings(e)) => {
                        println!("{e}")
                    }
                    Ok(ReplOutput::Write(msg)) => eprintln!("{msg}"),
                    Err(e) => {
                        print_error(line.as_str(), e);
                    }
                }
            }

            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {err}");
                break;
            }
        }
    }

    rl.save_history(HISTORY_FILE)
}


fn print_error(input: &str, e: ReplError) {
    use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

    let mut colors = ColorGenerator::new();
    let a = colors.next();

    match e {
        ReplError::ParseError => eprintln!("Parse Error"),
        ReplError::EvalError(eval_error) => {
            let Some(source_location) = eval_error.location else {
                eprintln!("EvalError");
                return;
            };

            let builder = Report::build(ReportKind::Error, "REPL", source_location.start);
            
            let builder = builder.with_code(3);

            let builder = builder.with_message(match eval_error.reason {
                EvalErrorReason::KindError(_) => "Kind Error",
                EvalErrorReason::TypeError(_, _) => "Type Error",
                EvalErrorReason::CollectionTypeError(_) => "Collection Type Error",
                EvalErrorReason::CastError(_, _) => "Cast Error",
                EvalErrorReason::UnknownIdentifier(_) => "UnknownIdentifier",
                EvalErrorReason::InvalidNumber(_) => "Invalid Number",
                EvalErrorReason::MathDivisionByZero => "Division By Zero",
                EvalErrorReason::KeyNotDefined(_, _) => "Key not defined",
                EvalErrorReason::OutOfBound(_, _) => "Out of bounds error",
                EvalErrorReason::IntegerOverflow => "Integer overflow",
                EvalErrorReason::UnknownFunction(_) => "Unknown function",
                EvalErrorReason::PatternError(_) => "Pattern Fail",
                EvalErrorReason::PatternExhaustionError(_) => "Non exhaustive Pattern matching",
            });

            let builder = builder.with_label(
                Label::new(("REPL", source_location.start..(source_location.end)))
                    .with_message("The error occured here")
                    .with_color(a),
            );
            

            builder.finish()
            .print(("REPL", Source::from(input)))
            .unwrap();
        },
        ReplError::MatchError(pattern_fail) => {eprintln!("Match Failed"); dbg!(pattern_fail);},
        ReplError::TopologyError(topology_error) => {eprintln!("Topoloy Error"); dbg!(topology_error);},
        ReplError::TransformError => eprintln!("Error During Transformation"),
    }
}