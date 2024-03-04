use crate::literal::single_string_literal;
use chumsky::error::Error;
use chumsky::extra;
use chumsky::prelude::just;
use chumsky::prelude::Rich;
use chumsky::text::unicode::ident;

use damasc_lang::identifier::Identifier;

use chumsky::Parser;

pub fn single_identifier<'s>(
) -> impl Parser<'s, &'s str, Identifier<'s>, extra::Err<Rich<'s, char>>> {
    ident()
        .try_map(move |c: &'s str, span| {
            if matches!(
                c,
                "where" | "into" | "limit" | "with" | "fn" | "match" | "if" | "else" | "for" | "in"
            ) {
                Err(Error::<&'s str>::expected_found(None, None, span))
            } else {
                Ok(c)
            }
        })
        .or(just("#").ignore_then(ident()))
        .map(Identifier::new)
        .or(just("#")
            .ignore_then(single_string_literal())
            .map(Identifier::new_cow))
}
