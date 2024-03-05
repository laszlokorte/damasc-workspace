#![feature(iter_intersperse)]
use crate::error::print_error;
use ariadne::Color;
use ariadne::ColorGenerator;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::Source;
use chumsky::Parser;
use damasc_grammar::repl::single_command;

use damasc_repl::{io::ReplOutput, state::State};
use rustyline::{error::ReadlineError, Editor};

mod error;

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

                let repl_parser = single_command();
                let cmd = match repl_parser.parse(&line).into_result() {
                    Ok(cmd) => cmd,
                    Err(errs) => {
                        let mut colors = ColorGenerator::new();
                        errs.into_iter().for_each(|e| {
                            Report::build(ReportKind::Error, "Inline", e.span().start)
                                .with_code("Parse Error")
                                .with_message(e.to_string())
                                .with_label(
                                    Label::new(("Inline", e.span().into_range()))
                                        .with_message(e.reason().to_string())
                                        .with_color(Color::Red),
                                )
                                .with_labels(e.contexts().map(|(label, span)| {
                                    Label::new(("Inline", span.into_range()))
                                        .with_message(format!("while parsing this {}", label))
                                        .with_color(colors.next())
                                }))
                                .finish()
                                .print(("Inline", Source::from(&line)))
                                .unwrap()
                        });
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
