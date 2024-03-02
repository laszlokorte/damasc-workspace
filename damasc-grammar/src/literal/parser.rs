use chumsky::extra;
use chumsky::prelude::Rich;
use std::borrow::Cow;

use chumsky::Parser;

use damasc_lang::literal::Literal;

use chumsky::prelude::*;

pub fn single_string_literal<'s>(
) -> Boxed<'s, 's, &'s str, Cow<'s, str>, extra::Err<Rich<'s, char>>> {
    let escape = just('\\')
        .then(choice((
            just('\\'),
            just('/'),
            just('"'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('u').ignore_then(text::digits(16).exactly(4).to_slice().validate(
                |digits, e, emitter| {
                    char::from_u32(u32::from_str_radix(digits, 16).unwrap()).unwrap_or_else(|| {
                        emitter.emit(Rich::custom(e.span(), "invalid unicode character"));
                        '\u{FFFD}' // unicode replacement character
                    })
                },
            )),
        )))
        .ignored()
        .boxed();

    let string = none_of("\\\"")
        .ignored()
        .or(escape)
        .repeated()
        .to_slice()
        .map(Cow::Borrowed)
        .delimited_by(just('"'), just('"'))
        .labelled("string")
        .as_context()
        .boxed();

    string.boxed()
}

pub fn single_literal<'s>() -> Boxed<'s, 's, &'s str, Literal<'s>, extra::Err<Rich<'s, char>>> {
    let integer = just('-')
        .or_not()
        .then(text::int(10))
        .to_slice()
        .map(Cow::Borrowed)
        .boxed();

    let boolean = choice((
        just("true").to(Literal::Boolean(true)).labelled("true"),
        just("false").to(Literal::Boolean(false)).labelled("false"),
    ));

    choice((
        just("null").to(Literal::Null).labelled("null"),
        boolean,
        integer.map(Literal::Number),
        single_string_literal().map(Literal::String),
    ))
    .boxed()
}
