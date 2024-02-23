#![feature(iter_array_chunks)]

use damasc_lang::parser;

#[test]
fn test_value_parsing() {
    let lines = include_str!("./examples_values.txt").lines();

    for (number, line) in lines.filter(|l| !l.is_empty()).enumerate() {
        assert!(
            parser::value::single_value(line).is_some(),
            "could not parse line {}",
            number + 1
        );
    }
}
