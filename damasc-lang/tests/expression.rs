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
    let lines = include_str!("./examples_expression_pairs.txt")
        .lines()
        .enumerate();

    for [(line_a, a), (line_b, b), (_, sep)] in lines.array_chunks() {
        assert_eq!(sep, "---");
        let Some(a) = parser::expression::expression_many1_all_consuming(a) else {
            eprintln!("Parse error at line {}: {}", line_a + 1, a);
            unreachable!("Parse error");
        };

        let Some(b) = parser::expression::expression_many1_all_consuming(b) else {
            eprintln!("Parse error at line {}: {}", line_b + 1, b);
            unreachable!("Parse error");
        };

        for (a, b) in a.expressions.into_iter().zip(b.expressions.into_iter()) {
            let eval = Evaluation::default();
            let Ok(res_a) = eval.eval_expr(&a) else {
                eprintln!("Evaluation error at line {}: {}", line_a + 1, a);
                unreachable!("Evaluation error");
            };
            let Ok(res_b) = eval.eval_expr(&b) else {
                eprintln!("Evaluation error at line {}: {}", line_b + 1, b);
                unreachable!("Evaluation error");
            };

            assert_eq!(res_a, res_b, "on line {} and {}", line_a + 1, line_b + 1);
        }
    }
}
