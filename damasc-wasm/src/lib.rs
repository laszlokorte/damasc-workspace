use damasc_repl::{parser, state::State};

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen(module = "/js/damasc.js")]
extern "C" {
    fn show_error(stmt: &str, error: &str);
    fn show_result(stmt: &str, result: &str);
}

#[wasm_bindgen]
pub struct WasmRepl {
    state: Box<State<'static, 'static, 'static>>,
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
        let cmd = match parser::command_all_consuming(input) {
            Ok(cmd) => cmd,
            Err(e) => {
                return show_error(input, &e);
            }
        };

        match self.state.eval(cmd) {
            Ok(r) => return show_result(input, &format!("{r}")),
            Err(e) => return show_error(input, &format!("Error: {e:?}")),
        }
    }
}
