#![feature(assert_matches)]

use std::assert_matches::assert_matches;

use damasc_join::parser;
use damasc_lang::identifier::Identifier;

#[test]
fn read_bag_bundle() {
    let bundle_string = include_str!("./example_bundle.txt");

    let Some(bundle) = parser::bag_bundle_all_consuming(bundle_string) else {
        unreachable!("bundle parse error");
    };

    assert_eq!(bundle.bags.len(), 3);
    let Some(b) = bundle.bags.get(&Identifier::new("foo")) else {
        unreachable!("bag foo does not exists");
    };
    assert_eq!(b.len(), 4);

    let Some(b) = bundle.bags.get(&Identifier::new("bar")) else {
        unreachable!("bag bar does not exists");
    };
    assert_eq!(b.len(), 4);

    let Some(b) = bundle.bags.get(&Identifier::new("woop")) else {
        unreachable!("bag woop does not exists");
    };
    assert_eq!(b.len(), 4);
}

#[test]
fn read_bag_bundle_duplicates() {
    let bundle_string = include_str!("./example_bundle_duplicate.txt");

    assert_matches!(parser::bag_bundle_all_consuming(bundle_string), None);
}

#[test]
fn read_bag_bundle_empty() {
    let bundle_string = include_str!("./example_bundle_empty.txt");

    assert_matches!(parser::bag_bundle_all_consuming(bundle_string), Some(_));
}
