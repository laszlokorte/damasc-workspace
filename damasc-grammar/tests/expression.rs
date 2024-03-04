use chumsky::Parser;
use damasc_grammar::expression::single_expression;

use ariadne::Color;
use ariadne::ColorGenerator;
use ariadne::Fmt;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::Source;


#[test]
fn expression_parsing() {
    let lines = include_str!("./examples_expressions.txt").lines();
    let expr = single_expression();
    let mut colors = ColorGenerator::new();
    let mut fails = 0;

    for (ln, line) in lines.enumerate() {
        match expr.parse(line).into_result() {
            Ok(_) => continue,
            Err(errs) => {
                fails+=1;
                errs.into_iter().for_each(|e| {
                        Report::build(ReportKind::Error, "Inline", e.span().start)
                            .with_code("Parse Error")
                            .with_message(e.to_string())
                            .with_label(
                                Label::new(("Inline", e.span().into_range()))
                                    .with_message(e.reason().to_string())
                                    .with_color(Color::Red),
                            )
                            .with_labels(e.contexts().map(|(label, span)| {
                                Label::new(("Inline", span.into_range()))
                                    .with_message(format!("while parsing this {}", label))
                                    .with_color(colors.next())
                            }))
                            .finish()
                            .print(("Inline", Source::from(&line)))
                            .unwrap()
                    });
            }
        }
    }

    assert_eq!(0, fails);
}
