
use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;
use damasc_lang::identifier::Identifier;
use damasc_lang::syntax::pattern::Pattern;
use damasc_lang::syntax::pattern::PatternBody;

pub fn single_pattern<'a>(
) -> impl Parser<'a, &'a str, Pattern<'a>, extra::Err<Rich<'a, char>>> {
    let identifier = chumsky::text::ident()
        .map(|c| Pattern::new(PatternBody::<'a>::Identifier(Identifier::new(c))));

    identifier
}
