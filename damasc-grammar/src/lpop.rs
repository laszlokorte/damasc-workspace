use lalrpop_util::lalrpop_mod;

pub mod error;
pub mod tokens;
pub mod lexer;

lalrpop_mod!(pub grammar, "/lpop/grammar.rs");
