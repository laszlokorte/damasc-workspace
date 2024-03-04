use chumsky::extra;
use chumsky::prelude::Rich;
use damasc_lang::value_type::ValueType;
use std::borrow::Cow;

use chumsky::Parser;

use damasc_lang::literal::Literal;

use chumsky::prelude::*;

pub fn single_string_literal<'s>(
) -> impl Parser<'s, &'s str, Cow<'s, str>, extra::Err<Rich<'s, char>>> {
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

    none_of("\\\"")
        .ignored()
        .or(escape)
        .repeated()
        .to_slice()
        .map(Cow::Borrowed)
        .delimited_by(just('"'), just('"'))
        .labelled("string")
        .as_context()
}

pub fn single_type_literal<'s>() -> impl Parser<'s, &'s str, ValueType, extra::Err<Rich<'s, char>>>
{
    choice((
        just("Type").to(ValueType::Type).labelled("Type"),
        just("Null").to(ValueType::Null).labelled("Null"),
        just("Boolean").to(ValueType::Boolean).labelled("Boolean"),
        just("Integer").to(ValueType::Integer).labelled("Integer"),
        just("Array").to(ValueType::Array).labelled("Array"),
        just("Object").to(ValueType::Object).labelled("Object"),
        just("String").to(ValueType::String).labelled("String"),
        just("Lambda").to(ValueType::Lambda).labelled("Lambda"),
    ))
    .boxed()
}

pub fn single_literal<'s>() -> impl Parser<'s, &'s str, Literal<'s>, extra::Err<Rich<'s, char>>> {
    let integer = just('-')
        .or_not()
        .then(text::int(10))
        .to_slice()
        .map(Cow::Borrowed)
        .map(Literal::Number)
        .boxed();

    let boolean = choice((
        just("true").to(true).labelled("true"),
        just("false").to(false).labelled("false"),
    ))
    .map(Literal::Boolean)
    .boxed();

    let value_type = single_type_literal().map(Literal::Type).boxed();

    let null = just("null").to(Literal::Null).labelled("null");

    choice((
        null,
        boolean,
        integer,
        value_type,
        single_string_literal().map(Literal::String),
    ))
}
