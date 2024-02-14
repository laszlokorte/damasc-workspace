#![feature(iter_array_chunks)]

use damasc_lang::parser;

#[test]
fn test_value_parsing() {
    let lines = include_str!("./examples_values.txt").lines();

    for line in lines.filter(|l| l.len() > 0) {
        dbg!(line);
        assert!(parser::value::single_value(line).is_some());
    }
}
