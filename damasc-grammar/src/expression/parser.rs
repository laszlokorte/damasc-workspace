use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;
use damasc_lang::identifier::Identifier;
use damasc_lang::syntax::expression::Expression;
use damasc_lang::syntax::expression::ExpressionBody;

pub fn single_expression<'a>(
) -> impl Parser<'a, &'a str, Expression<'a>, extra::Err<Rich<'a, char>>> {
    let identifier = chumsky::text::ident()
        .map(|c| Expression::new(ExpressionBody::<'a>::Identifier(Identifier::new(c))));

    identifier
}
