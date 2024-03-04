use damasc_lang::syntax::pattern::Pattern;
use damasc_query::capture::MultiCapture;
use damasc_lang::syntax::pattern::PatternBody;
use damasc_lang::syntax::pattern::PatternSet;
use damasc_lang::syntax::expression::ExpressionSet;
use damasc_lang::identifier::Identifier;
use damasc_lang::syntax::expression::Expression;
use damasc_lang::literal::Literal;
use damasc_lang::syntax::expression::ExpressionBody;
use damasc_query::predicate::MultiPredicate;
use crate::pattern::pattern_set_non_empty;
use crate::expression::single_expression;
use chumsky::prelude::any;
use chumsky::prelude::skip_then_retry_until;
use chumsky::prelude::end;
use chumsky::prelude::via_parser;
use chumsky::prelude::just;
use crate::expression::expression_set_non_empty;

use damasc_query::projection::MultiProjection;


use damasc_query::transformation::Transformation;
use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

pub fn single_transformation<'s, 'a,'b>() -> impl Parser<'s, &'s str, Transformation<'a, 'b>, extra::Err<Rich<'s, char>>> {
    expression_set_non_empty()
                    .delimited_by(
                        just('{'),
                        just('}')
                            .ignored()
                            .recover_with(via_parser(end()))
                            .recover_with(skip_then_retry_until(any().ignored(), end())),
                    ).then(projection()).map(move |(bag, projection)| {
    	Transformation{
    		bag,
    		projection,
    	}
    })
}


fn projection<'s, 'a>() -> impl Parser<'s, &'s str, MultiProjection<'a>, extra::Err<Rich<'s, char>>> {
	let map = just("map").ignore_then(pattern_set_non_empty()).or_not();
	let guard = just("where").ignore_then(single_expression()).or_not();
	let into = just("into").ignore_then(expression_set_non_empty()).or_not();

	map.then(guard).then(into).map(move |((patterns, guard), proj)| {
		let pats = patterns.unwrap_or(PatternSet {
            patterns: vec![Pattern::new(PatternBody::Discard)],
        });
        let auto_named_pats = PatternSet {
            patterns: pats
                .patterns
                .into_iter()
                .enumerate()
                .map(|(i, p)| {
                    Pattern::new(PatternBody::Capture(
                        Identifier::new_owned(format!("${i}")),
                        Box::new(p.deep_clone()),
                    ))
                })
                .collect(),
        };
        let auto_projection = ExpressionSet {
            expressions: (0..auto_named_pats.patterns.len())
                .map(|i| {
                    Expression::new(ExpressionBody::Identifier(Identifier::new_owned(
                        format!("${i}"),
                    )))
                })
                .collect(),
        };
        MultiProjection {
            predicate: MultiPredicate {
                capture: MultiCapture {
                    patterns: auto_named_pats,
                },
                guard: guard.unwrap_or(Expression::new(ExpressionBody::Literal(
                    Literal::Boolean(true),
                ))),
            },
            projections: proj.unwrap_or(auto_projection),
        }
	})
}