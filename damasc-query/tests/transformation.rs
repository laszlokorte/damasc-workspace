#![feature(iter_array_chunks)]

use damasc_lang::{runtime::{evaluation::{self, Evaluation}, env::Environment}, value::Value};
use damasc_query::{parser, iter::MultiProjectionIterator};
use itertools::Itertools;

#[test]
fn test_transformation() {
    let lines = include_str!("./transformation_examples.txt").lines();

    for [trans,res, delimiter] in lines.array_chunks() {
        assert_eq!(delimiter, "---");
        let Some(transformation) = parser::single_transformation(trans) else {
            unreachable!("Transformation parse error");
        };

        let Some(result) = parser::single_bag(res) else {
            unreachable!("Transformation parse error");
        };

        let env = Environment::default();
        let evaluation = Evaluation::default();
        let Some(result_values) = result.expressions.iter().map(|e| evaluation.eval_expr(e)).collect::<Result<Vec<Value>, _>>().ok() else {
            unreachable!("Result Eval error");
        };

        let iter = transformation.bag.expressions.iter().filter_map(|e| evaluation.eval_expr(e).ok());
        let trans_iterator = MultiProjectionIterator::new(env, transformation.projection, iter);
        
        let transform_result = trans_iterator.flatten().flatten().collect::<Vec<_>>();

        assert_eq!(transform_result, result_values);
    }


}