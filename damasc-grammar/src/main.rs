fn main() {
    #[cfg(feature = "assignment")]
    {
        use ariadne::Color;
        use ariadne::Fmt;
        use ariadne::Label;
        use ariadne::Report;
        use ariadne::ReportKind;
        use ariadne::Source;
        use chumsky::Parser;
        use damasc_grammar::assignment::parser::single_assignment;

        for src in ["foo =   bar", "abc = 42", "foo bar"] {
            let result = single_assignment().parse(src.trim()).into_result();
            match result {
                Ok(expr) => println!("{}: {}", "Success".fg(Color::Green), expr),
                Err(errs) => {
                    errs.into_iter().for_each(|e| {
                        Report::build(ReportKind::Error, "Inline", e.span().start)
                            .with_code("Parse Error")
                            .with_message(e.to_string())
                            .with_label(
                                Label::new(("Inline", e.span().into_range()))
                                    .with_message(e.reason().to_string())
                                    .with_color(Color::Red),
                            )
                            .finish()
                            .print(("Inline", Source::from(&src)))
                            .unwrap()
                    });
                }
            };
        }
    }
}
