
use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

pub fn single_join<'a>(
) -> impl Parser<'a, &'a str, _<'a>, extra::Err<Rich<'a, char>>> {
    todo!();
}
