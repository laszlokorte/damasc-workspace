use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

pub fn single_join<'a, TODO>() -> impl Parser<'a, &'a str, TODO<'a>, extra::Err<Rich<'a, char>>> {
    todo!();
}
