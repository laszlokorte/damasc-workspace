#![feature(assert_matches)]

use core::assert_matches::assert_matches;
use damasc_lang::parser;
use damasc_lang::runtime::env::Environment;
use damasc_lang::runtime::matching::Matcher;

#[test]
fn test_matching_fail() {
    let lines = include_str!("./examples_mismatch.txt").lines();
    for line in lines {
        let Some(assignment) = parser::assignment::assignment_set1_all_consuming(line) else {
            unreachable!("Can not parse assignments");
        };

        let env = Environment::new();
        let matcher = Matcher::new(&env);

        assert_matches!(matcher.eval_assigment_set(assignment), Err(_));
    }
}
