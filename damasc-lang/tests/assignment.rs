#![feature(assert_matches)]

use std::assert_matches::assert_matches;

use damasc_lang::parser;
use damasc_lang::runtime::matching::Matcher;

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
    
    for line in lines {
        if let Some(assignment) = parser::assignment::assignment_set1_all_consuming(line) {
            let Ok(sorted) = assignment.sort_topological() else {
                unreachable!("Unexpected cyclic dependency in assignments");
            };

            let matcher = Matcher::default();
            
            assert_matches!(matcher.eval_assigment_set(sorted), Ok(_));
        }
    }
}