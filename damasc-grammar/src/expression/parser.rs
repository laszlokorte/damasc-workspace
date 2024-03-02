use crate::util::meta_to_location;
use damasc_lang::syntax::expression::ExpressionBody;
use damasc_lang::syntax::expression::Expression;

use crate::literal::parser::single_literal;

use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

use chumsky::prelude::*;

pub fn single_expression<'s>() -> Boxed<'s, 's, &'s str, Expression<'s>, extra::Err<Rich<'s, char>>> {
    recursive(|_expression| {
        // let object = ...;
        // let array = ...;
        // let abstraction = ...;
        // let maching = ...;
        // let condition = ...;

        let literal = single_literal();

        literal.map_with(|l, meta| Expression::new_with_location(ExpressionBody::Literal(l), meta_to_location(meta)))
    })
    .boxed()
}
