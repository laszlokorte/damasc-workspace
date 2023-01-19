#![feature(assert_matches)]

use damasc_lang::{parser::{value::value_bag, pattern::full_pattern, expression::single_expression}, runtime::env::Environment, value::Value};
use damasc_query::{predicate::{Predicate, PredicateError}, projection::ProjectionError, capture::Capture};
use damasc_query::iter::ProjectionIterator;
use damasc_query::projection::Projection;
use damasc_query::iter::PredicateIterator;
use std::assert_matches::assert_matches;


#[test]
fn test_predicate_iteration() {
    let values = include_str!("./example_values.txt");
    let Some(bag) = value_bag(values) else {
        unreachable!("Values could not be read.");
    };
    let Some(pattern) = full_pattern("[_,_]") else {
        unreachable!("Pattern parse error");
    };
    let Some(guard) = single_expression("true") else {
        unreachable!("Pattern parse error");
    };

    let pred = Predicate {
        capture: Capture { pattern },
        guard
    };

    let iter = bag.values.iter();
    let env = Environment::default();

    let pred_iter = PredicateIterator::new(env, pred, iter);

    for v in pred_iter.clone() {
        dbg!(&v);
    }

    assert_eq!(bag.values.len(), 17);
    assert_eq!(pred_iter.count(), 4);

}

#[test]
fn test_projection_constant_iteration() {
    let values = include_str!("./example_values.txt");
    let Some(bag) = value_bag(values) else {
        unreachable!("Values could not be read.");
    };
    let Some(pattern) = full_pattern("[_,_]") else {
        unreachable!("Pattern parse error");
    };
    let Some(guard) = single_expression("true") else {
        unreachable!("Guard parse error");
    };
    let Some(proj_expression) = single_expression("42") else {
        unreachable!("Projection parse error");
    };

    let predicate = Predicate {
        capture: Capture { pattern },
        guard
    };

    let projection = Projection {
        predicate,
        projection: proj_expression,
    };

    let iter = bag.values.iter();
    let env = Environment::default();

    let pred_iter = ProjectionIterator::new(env, projection, iter);

    for v in pred_iter.clone() {
        assert_matches!(v, Ok(Value::Integer(42)));
    }

    assert_eq!(bag.values.len(), 17);
    assert_eq!(pred_iter.count(), 4);

}


#[test]
fn test_projection_dynamic_iteration() {
    let values = include_str!("./example_values.txt");
    let Some(bag) = value_bag(values) else {
        unreachable!("Values could not be read.");
    };
    let Some(pattern) = full_pattern("[x,y]") else {
        unreachable!("Pattern parse error");
    };
    let Some(guard) = single_expression("x != y") else {
        unreachable!("Guard parse error");
    };
    let Some(proj_expression) = single_expression("x+y") else {
        unreachable!("Projection parse error");
    };

    let predicate = Predicate {
        capture: Capture { pattern },
        guard
    };

    let projection = Projection {
        predicate,
        projection: proj_expression,
    };

    let iter = bag.values.iter();
    let env = Environment::default();

    let pred_iter = ProjectionIterator::new(env, projection, iter);

    for v in pred_iter.clone() {
        assert_matches!(v, Ok(Value::Integer(65)));
    }

    assert_eq!(bag.values.len(), 17);
    assert_eq!(pred_iter.count(), 3);

}


#[test]
fn test_projection_eval_error_iteration() {
    let values = include_str!("./example_values.txt");
    let Some(bag) = value_bag(values) else {
        unreachable!("Values could not be read.");
    };
    let Some(pattern) = full_pattern("[x,y]") else {
        unreachable!("Pattern parse error");
    };
    let Some(guard) = single_expression("true") else {
        unreachable!("Guard parse error");
    };
    let Some(proj_expression) = single_expression("z+y") else {
        unreachable!("Projection parse error");
    };

    let predicate = Predicate {
        capture: Capture { pattern },
        guard
    };

    let projection = Projection {
        predicate,
        projection: proj_expression,
    };

    let iter = bag.values.iter();
    let env = Environment::default();

    let pred_iter = ProjectionIterator::new(env, projection, iter);

    for v in pred_iter.clone() {
        assert_matches!(v, Err(ProjectionError::EvalError));
    }

    assert_eq!(bag.values.len(), 17);
    assert_eq!(pred_iter.count(), 4);

}


#[test]
fn test_projection_guard_error_iteration() {
    let values = include_str!("./example_values.txt");
    let Some(bag) = value_bag(values) else {
        unreachable!("Values could not be read.");
    };
    let Some(pattern) = full_pattern("[x,y]") else {
        unreachable!("Pattern parse error");
    };
    let Some(guard) = single_expression("x > z") else {
        unreachable!("Guard parse error");
    };
    let Some(proj_expression) = single_expression("x+y") else {
        unreachable!("Projection parse error");
    };

    let predicate = Predicate {
        capture: Capture { pattern },
        guard
    };

    let projection = Projection {
        predicate,
        projection: proj_expression,
    };

    let iter = bag.values.iter();
    let env = Environment::default();

    let pred_iter = ProjectionIterator::new(env, projection, iter);

    for v in pred_iter.clone() {
        assert_matches!(v, Err(ProjectionError::PredicateError(PredicateError::GuardError)));
    }

    assert_eq!(bag.values.len(), 17);
    assert_eq!(pred_iter.count(), 4);

}


#[test]
fn test_projection_pattern_error_iteration() {
    let values = include_str!("./example_values.txt");
    let Some(bag) = value_bag(values) else {
        unreachable!("Values could not be read.");
    };
    let Some(pattern) = full_pattern(r"{[x]: 42}") else {
        unreachable!("Pattern parse error");
    };
    let Some(guard) = single_expression("x > z") else {
        unreachable!("Guard parse error");
    };
    let Some(proj_expression) = single_expression("x+y") else {
        unreachable!("Projection parse error");
    };

    let predicate = Predicate {
        capture: Capture { pattern },
        guard
    };

    let projection = Projection {
        predicate,
        projection: proj_expression,
    };

    let iter = bag.values.iter();
    let env = Environment::default();

    let pred_iter = ProjectionIterator::new(env, projection, iter);

    for v in pred_iter.clone() {
        assert_matches!(v, Err(ProjectionError::PredicateError(PredicateError::PatternError)));
    }

    assert_eq!(bag.values.len(), 17);
    assert_eq!(pred_iter.count(), 1);

}