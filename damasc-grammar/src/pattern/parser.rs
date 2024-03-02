use crate::identifier::parser::single_identifier;
use crate::util::meta_to_location;
use damasc_lang::syntax::pattern::PatternBody;
use damasc_lang::syntax::pattern::Pattern;

use crate::literal::parser::single_literal;

use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

use chumsky::prelude::*;

pub fn single_pattern<'s>() -> impl Parser<'s, &'s str, Pattern<'s>, extra::Err<Rich<'s, char>>> {
    recursive(|_pattern| {
        let literal = single_literal().boxed().map_with(|l, meta| Pattern::new_with_location(PatternBody::Literal(l), meta_to_location(meta)))
        .labelled("literal")
        .as_context();

        let identifier = single_identifier().boxed().map_with(|i, meta| Pattern::new_with_location(PatternBody::Identifier(i), meta_to_location(meta)))
        .labelled("identifier")
        .as_context();



        choice((
            literal,
            identifier
        ))
    })
}
