#![feature(assert_matches)]

use std::assert_matches::assert_matches;

use damasc_lang::parser;

#[test]
fn test_topology_fail() {
    let lines = include_str!("./examples_topology_fail.txt").lines();
    for line in lines {

        let Some(assignment) = parser::assignment::assignment_set1_all_consuming(line) else {
            unreachable!("Can not parse assignments");
        };

        assert_matches!(assignment.sort_topological(), Err(_));
    }
}