use chumsky::error::Error;
use chumsky::extra;
use chumsky::prelude::Rich;
use chumsky::text::unicode::ident;
use chumsky::Boxed;
use damasc_lang::identifier::Identifier;

use chumsky::Parser;

pub fn single_identifier<'s>() -> Boxed<'s, 's, &'s str, Identifier<'s>, extra::Err<Rich<'s, char>>>
{
    ident()
        .try_map(move |c: &'s str, span| {
            if matches!(
                c,
                "where" | "into" | "limit" | "with" | "fn" | "match" | "if" | "else"
            ) {
                Err(Error::<&'s str>::expected_found(None, None, span))
            } else {
                Ok(c)
            }
        })
        .map(Identifier::new)
        .boxed()
}
