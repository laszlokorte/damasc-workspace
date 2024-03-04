
fn main() {
    #[cfg(feature = "assignment")]
    {
        use ariadne::Color;
        use ariadne::ColorGenerator;
        use ariadne::Fmt;
        use ariadne::Label;
        use ariadne::Report;
        use ariadne::ReportKind;
        use ariadne::Source;
        use chumsky::Parser;
        use damasc_grammar::assignment::assignment_set;


        let mut colors = ColorGenerator::new();

        for src in [
            "foo =   bar; x=5;y=[1,2,3]",
            "abc = 42",
            "foo bar; x=5;y=[1,2,3]",
            "42 = 2",
            "foo = (a",
            "foo = )a",
            "foo = (a]",
            "y = [x,y,z for x in y]",
            "y = [x,y,z for match ^(x+x) in y]",
            "y = [x,y,z for match {x,y} in y]",
            "y = [x,y,z for match {x ...y} in y]",
        ] {
            let result = assignment_set().parse(src).into_result();
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
                            .with_labels(e.contexts().map(|(label, span)| {
                                Label::new(("Inline", span.into_range()))
                                    .with_message(format!("while parsing this {}", label))
                                    .with_color(colors.next())
                            }))
                            .finish()
                            .print(("Inline", Source::from(&src)))
                            .unwrap()
                    });
                }
            };
        }
    }

    #[cfg(feature = "value")]
    {
        use ariadne::Color;
        use ariadne::ColorGenerator;
        use ariadne::Fmt;
        use ariadne::Label;
        use ariadne::Report;
        use ariadne::ReportKind;
        use ariadne::Source;
        use chumsky::Parser;
        use damasc_grammar::value::single_value;

        let mut colors = ColorGenerator::new();

        for src in [
            "42",
            "true",
            "false",
            "[23,42]",
            "{\"x\": 15}",
            "{\"x\": 15, x: 15}",
            "{x: 15}",
            "null",
            "[[{\"\" x}]]",
            "{\"foo\":[,]}",
            "    {  \"foo\"  :  [ , ]}",
            "[  . ]",
            "{ \"foo\": . }",
            "[{ \"foo\": . }]",
        ] {
            let result = single_value().parse(src).into_result();
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
                            .with_labels(e.contexts().scan(0, |offset, (label, span)| {
                                let label = Label::new((
                                    "Inline",
                                    (span.start + *offset)..(span.end + *offset),
                                ))
                                .with_message(format!("while parsing this {}", label))
                                .with_color(colors.next());

                                Some(label)
                            }))
                            .finish()
                            .print(("Inline", Source::from(&src)))
                            .unwrap()
                    });
                }
            };
        }
    }
}
