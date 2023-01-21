#![feature(iter_array_chunks)]

use damasc_lang::{parser, runtime::evaluation::Evaluation};

#[test]
fn test_expression_parsing() {
    let lines = include_str!("./examples_expression_pairs.txt").lines();
    
    for line in lines.filter(|l| l != &"---") {
        assert!(parser::expression::expression_many1_all_consuming(line).is_some());
    }
}
#[test]
fn test_expression_evaluation() {
    let lines = include_str!("./examples_expression_pairs.txt").lines();
    
    for [a,b,sep] in lines.array_chunks() {
        assert_eq!(sep, "---");
        let Some(a) = parser::expression::expression_many1_all_consuming(a) else {
            dbg!(a);
            unreachable!("Parse error");
        };

        let Some(b) = parser::expression::expression_many1_all_consuming(b) else {
            dbg!(a);
            unreachable!("Parse error");
        };

        for (a,b) in a.expressions.into_iter().zip(b.expressions.into_iter()) {
            let eval = Evaluation::default();
            let Ok(res_a) = eval.eval_expr(&a) else {
                unreachable!("Evaluation error");
            };
            let Ok(res_b) = eval.eval_expr(&b) else {
                unreachable!("Evaluation error");
            };

            assert_eq!(res_a, res_b);
        }
    }
}