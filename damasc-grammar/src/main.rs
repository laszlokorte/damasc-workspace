use ariadne::Color;
use ariadne::Fmt;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::Source;
use chumsky::Parser;
use damasc_grammar::expression::parser::single_expression;

fn main() {
    for src in ["abc", "+#,"] {
        let result = single_expression().parse(src.trim()).into_result();
        match result {
            Ok(expr) => println!("{}: {}", "Success".fg(Color::Green), expr),
            Err(errs) => {
                errs.into_iter().for_each(|e| {
                    Report::build(ReportKind::Error, (), e.span().start)
                        .with_message(e.to_string())
                        .with_label(
                            Label::new(e.span().into_range())
                                .with_message(e.reason().to_string())
                                .with_color(Color::Red),
                        )
                        .finish()
                        .print(Source::from(&src))
                        .unwrap()
                });
            }
        };
    }
}
