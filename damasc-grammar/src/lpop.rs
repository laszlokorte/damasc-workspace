use lalrpop_util::lalrpop_mod;

pub mod error;
pub mod lexer;
pub mod tokens;

lalrpop_mod!(pub grammar, "/lpop/grammar.rs");
