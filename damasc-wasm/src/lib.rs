#![feature(iter_intersperse)]

use crate::error::print_error;
use ariadne::Config;
use ariadne::Source;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::Label;
use chumsky::Parser;
use damasc_repl::{state::State};
use damasc_grammar::repl::single_command;
use wasm_bindgen::prelude::*;

mod error;

#[wasm_bindgen(module = "/js/damasc.js")]
extern "C" {
    fn show_error(stmt: &str, error: &str);
    fn show_result(stmt: &str, result: &str);
}

#[wasm_bindgen]
pub struct WasmRepl {
    state: Box<State<'static, 'static>>,
}

impl Default for WasmRepl {
    fn default() -> Self {
        Self {
            state: Box::new(State::new()),
        }
    }
}

#[wasm_bindgen]
impl WasmRepl {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            state: Box::new(State::new()),
        }
    }

    #[wasm_bindgen]
    pub fn eval(&mut self, input: &str) {

        let mut out_buffer = Vec::new();

        let parser = single_command();
        let cmd = match parser.parse(input).into_result() {
            Ok(cmd) => cmd,
            Err(errs) => {
                errs.into_iter().for_each(|e| {
                    Report::build(ReportKind::Error, "Inline", e.span().start)
                        .with_config(Config::default().with_color(false))
                        .with_code("Parse Error")
                        .with_message(e.to_string())
                        .with_label(
                            Label::new(("Inline", e.span().into_range()))
                                .with_message(e.reason().to_string()),
                        )
                        .with_labels(e.contexts().map(|(label, span)| {
                            Label::new(("Inline", span.into_range()))
                                .with_message(format!("while parsing this {}", label))
                        }))
                        .finish()
                        .write(("Inline", Source::from(&input)), &mut out_buffer)
                        .unwrap()
                });

                return match std::str::from_utf8(&out_buffer) {
                    Ok(r) => show_error(input, &r),
                    Err(_e) => show_error(input, "unexpected error"),
                }
            }
        };

        let mut out_buffer = Vec::new();
        match self.state.eval(cmd) {
            Ok(r) => show_result(input, &format!("{r}")),
            Err(eval_err) => {
                print_error(input, &eval_err, &mut out_buffer);
                match std::str::from_utf8(&out_buffer) {
                    Ok(r) => show_error(input, &r),
                    Err(_e) => show_error(input, "unexpected error"),
                }
            },
        }
    }
}
