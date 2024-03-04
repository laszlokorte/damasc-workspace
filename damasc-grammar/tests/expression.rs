#![feature(assert_matches)]
use chumsky::Parser;
use core::assert_matches::assert_matches;
use damasc_grammar::expression::single_expression;

#[test]
fn expression_parsing() {
    let lines = include_str!("./examples_expressions.txt").lines();
    let expr_parser = single_expression();

    for (ln, line) in lines.enumerate() {
        let parsed = expr_parser.parse(line).into_result();

        assert_matches!(parsed, Ok(_), "Failed to parse line {}: {}", ln + 1, line);
    }
}
