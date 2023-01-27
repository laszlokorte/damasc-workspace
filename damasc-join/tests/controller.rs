use damasc_join::{controller::Controller, parser, bag::Bag};
use damasc_lang::{identifier::Identifier, parser::value::single_value};

#[test]
fn test_join() {
    let mut controller = Controller::default();
    let mut foo_bag = Bag::default();
    foo_bag.insert(&single_value("22").unwrap());
    foo_bag.insert(&single_value("33").unwrap());
    foo_bag.insert(&single_value("44").unwrap());
    foo_bag.insert(&single_value("55").unwrap());
    foo_bag.insert(&single_value("66").unwrap());
    foo_bag.insert(&single_value("77").unwrap());
    foo_bag.insert(&single_value("77").unwrap());
    controller.storage.bags.insert(Identifier::new("foo"), foo_bag);
    let mut bar_bag = Bag::default();
    bar_bag.insert(&single_value("[77]").unwrap());
    bar_bag.insert(&single_value("[44]").unwrap());
    bar_bag.insert(&single_value("[66]").unwrap());
    bar_bag.insert(&single_value("[66,100]").unwrap());
    bar_bag.insert(&single_value("[]").unwrap());
    controller.storage.bags.insert(Identifier::new("bar"), bar_bag);
    let Some(join) = parser::join_all_consuming(include_str!("./example_join_simple.txt")) else {
        unreachable!("join parse error")
    };

    let results = controller.query(&join);

    assert_eq!(results.count(), 264);
}



#[test]
fn test_join_with_constant() {
    let mut controller = Controller::default();
    let mut foo_bag = Bag::default();
    foo_bag.insert(&single_value("22").unwrap());
    foo_bag.insert(&single_value("33").unwrap());
    foo_bag.insert(&single_value("44").unwrap());
    foo_bag.insert(&single_value("55").unwrap());
    foo_bag.insert(&single_value("66").unwrap());
    foo_bag.insert(&single_value("77").unwrap());
    foo_bag.insert(&single_value("77").unwrap());
    controller.storage.bags.insert(Identifier::new("foo"), foo_bag);
    let mut bar_bag = Bag::default();
    bar_bag.insert(&single_value("[77]").unwrap());
    bar_bag.insert(&single_value("[44]").unwrap());
    bar_bag.insert(&single_value("[66]").unwrap());
    bar_bag.insert(&single_value("[66,100]").unwrap());
    bar_bag.insert(&single_value("[]").unwrap());
    controller.storage.bags.insert(Identifier::new("bar"), bar_bag);
    let Some(join) = parser::join_all_consuming(include_str!("./example_join_simple_with_const.txt")) else {
        unreachable!("join parse error")
    };

    let results = controller.query(&join);
    // for r in results {
    //     println!("{}", r.environment);
    // }
    assert_eq!(results.count(), 120);
}