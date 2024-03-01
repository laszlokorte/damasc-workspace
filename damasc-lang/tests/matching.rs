#![feature(assert_matches)]

use core::assert_matches::assert_matches;
use damasc_lang::parser;
use damasc_lang::runtime::assignment::AssignmentEvaluation;
use damasc_lang::runtime::env::Environment;

#[test]
fn test_matching_fail() {
    let lines = include_str!("./examples_mismatch.txt").lines();
    for line in lines {
        let Some(assignment) = parser::assignment::assignment_set1_all_consuming(line) else {
            unreachable!("Can not parse assignments");
        };

        let env = Environment::default();
        let assignment_eval = AssignmentEvaluation::new(&env);

        assert_matches!(assignment_eval.eval_assigment_set(assignment), Err(_));
    }
}
