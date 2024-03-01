use std::borrow::Cow;
use chumsky::extra;
use chumsky::prelude::Rich;

use chumsky::Parser;

use damasc_lang::value::Value;

pub fn single_value<'a>(
) -> impl Parser<'a, &'a str, Value<'a,'a>, extra::Err<Rich<'a, char>>> {
    let identifier = chumsky::text::ident()
        .map(|c| Value::String(Cow::Borrowed(c)));

    identifier
}
