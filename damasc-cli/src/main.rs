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
                    Ok(ReplOutput::Write(_)) => println!("Write"),
                    Err(_) => eprintln!("ERR"),
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
