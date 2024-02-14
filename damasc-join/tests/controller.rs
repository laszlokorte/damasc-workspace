use damasc_join::{bag::Bag, controller::Controller, parser};
use damasc_lang::{identifier::Identifier, parser::value::single_value};
use itertools::Itertools;

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
    controller
        .storage
        .bags
        .insert(Identifier::new("foo"), foo_bag);
    let mut bar_bag = Bag::default();
    bar_bag.insert(&single_value("[77]").unwrap());
    bar_bag.insert(&single_value("[44]").unwrap());
    bar_bag.insert(&single_value("[66]").unwrap());
    bar_bag.insert(&single_value("[66,100]").unwrap());
    bar_bag.insert(&single_value("[]").unwrap());
    controller
        .storage
        .bags
        .insert(Identifier::new("bar"), bar_bag);
    let Some(join) = parser::join_all_consuming(include_str!("./example_join_simple.txt")) else {
        unreachable!("join parse error")
    };

    let all: Vec<_> = controller.query(&join).collect();

    dbg!(all.first());

    assert_eq!(all.len(), 264);
}

#[test]
fn test_join_with_insert() {
    let mut controller = Controller::default();
    let mut foo_bag = Bag::default();
    foo_bag.insert(&single_value("22").unwrap());
    foo_bag.insert(&single_value("33").unwrap());
    foo_bag.insert(&single_value("44").unwrap());
    foo_bag.insert(&single_value("55").unwrap());
    foo_bag.insert(&single_value("66").unwrap());
    foo_bag.insert(&single_value("77").unwrap());
    foo_bag.insert(&single_value("77").unwrap());
    controller
        .storage
        .bags
        .insert(Identifier::new("foo"), foo_bag);
    let mut bar_bag = Bag::default();
    bar_bag.insert(&single_value("[77]").unwrap());
    bar_bag.insert(&single_value("[44]").unwrap());
    bar_bag.insert(&single_value("[66]").unwrap());
    bar_bag.insert(&single_value("[66,100]").unwrap());
    bar_bag.insert(&single_value("[]").unwrap());
    controller
        .storage
        .bags
        .insert(Identifier::new("bar"), bar_bag);
    let Some(join) =
        parser::join_all_consuming(include_str!("./example_join_simple_with_insert.txt"))
    else {
        unreachable!("join parse error")
    };

    let all: Vec<_> = controller.query(&join).collect();

    for op in all {
        assert_eq!(op.insertions.len(), 3);
        assert_eq!(op.deletions.len(), 5);

        let x = op.deletions.iter().counts_by(|d| d.bag_id.name.clone());
        assert_eq!(x.get("foo").cloned().unwrap_or_default(), 3);
        assert_eq!(x.get("bar").cloned().unwrap_or_default(), 2);
    }
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
    controller
        .storage
        .bags
        .insert(Identifier::new("foo"), foo_bag);
    let mut bar_bag = Bag::default();
    bar_bag.insert(&single_value("[77]").unwrap());
    bar_bag.insert(&single_value("[44]").unwrap());
    bar_bag.insert(&single_value("[66]").unwrap());
    bar_bag.insert(&single_value("[66,100]").unwrap());
    bar_bag.insert(&single_value("[]").unwrap());
    controller
        .storage
        .bags
        .insert(Identifier::new("bar"), bar_bag);
    let Some(join) =
        parser::join_all_consuming(include_str!("./example_join_simple_with_const.txt"))
    else {
        unreachable!("join parse error")
    };

    let results = controller.query(&join);
    // for r in results {
    //     println!("{}", r.environment);
    // }
    assert_eq!(results.count(), 120);
}
