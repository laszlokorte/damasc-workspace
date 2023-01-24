#![feature(assert_matches)]

use std::assert_matches::assert_matches;

use damasc_join::parser;
#[test]
fn read_join() {

    assert_matches!(parser::join_all_consuming(include_str!("./example_join_full.txt")), Some(_));
    assert_matches!(parser::join_all_consuming(include_str!("./example_join_no_assign.txt")), Some(_));
    assert_matches!(parser::join_all_consuming(include_str!("./example_join_no_guard.txt")), Some(_));
    assert_matches!(parser::join_all_consuming(include_str!("./example_join_only_in.txt")), Some(_));
    assert_matches!(parser::join_all_consuming(include_str!("./example_join_only_out.txt")), Some(_));
    assert_matches!(parser::join_all_consuming(include_str!("./example_join_empty.txt")), Some(_));
    
}