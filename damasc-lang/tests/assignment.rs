#![feature(assert_matches)]

use damasc_lang::runtime::assignment::AssignmentEvaluation;
use damasc_lang::runtime::env::Environment;
use std::assert_matches::assert_matches;

use damasc_lang::parser;

#[test]
fn test_assignment_set_parsing() {
    let lines = include_str!("./examples_assignments.txt").lines();

    for line in lines {
        assert!(parser::assignment::assignment_set1_all_consuming(line).is_some());
    }
}

#[test]
fn test_assignment_set_evaluation() {
    let lines = include_str!("./examples_assignments.txt").lines();

    for (line_number, line) in lines.enumerate() {
        if let Some(assignment) = parser::assignment::assignment_set1_all_consuming(line) {
            let Ok(sorted) = assignment.sort_topological() else {
                unreachable!("Unexpected cyclic dependency in assignments");
            };

            let env = Environment::default();
            let assignment_eval = AssignmentEvaluation::new(&env);

            assert_matches!(
                assignment_eval.eval_assigment_set(sorted),
                Ok(_),
                "Failing assignment on line {}",
                line_number + 1
            );
        }
    }
}
