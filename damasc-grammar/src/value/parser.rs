use chumsky::extra;
use chumsky::prelude::Rich;
use std::borrow::Cow;

use chumsky::Parser;

use damasc_lang::value::Value;

use chumsky::prelude::*;


pub fn single_value<'s>() -> impl Parser<'s, &'s str, Value<'s,'s>, extra::Err<Rich<'s, char>>> {
    recursive(|value| {
        let value = value
        .labelled("value");

        let integer = just('-')
            .or_not()
            .then(text::int(10))
            .to_slice()
            .map(|s: &str| s.parse().unwrap()).labelled("integer")
            .boxed();

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
                        char::from_u32(u32::from_str_radix(digits, 16).unwrap()).unwrap_or_else(
                            || {
                                emitter.emit(Rich::custom(e.span(), "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            },
                        )
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
            .delimited_by(just('"'), just('"'))
            .labelled("string").as_context().boxed();

        let array = value
            .clone()
            .map(Cow::Owned)
            .separated_by(just(',').padded().recover_with(skip_then_retry_until(
                any().ignored(),
                one_of(",]").ignored(),
            )))
            .allow_trailing()
            .collect()
            .padded()
            .delimited_by(
                just('['),
                just(']')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            ).labelled("array").as_context()
            .boxed();

        let member = string.clone()
            .map(Cow::Borrowed).labelled("object_key").as_context().then_ignore(just(':').padded()).then(value
            .map(Cow::Owned).labelled("object_value").as_context());
        let object = member
            .clone()
            .separated_by(just(',').padded().recover_with(skip_then_retry_until(
                any().ignored(),
                one_of(",}").ignored(),
            )))
            .allow_trailing()
            .collect()
            .padded()
            .delimited_by(
                just('{'),
                just('}').ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            ).labelled("object").as_context()
            .boxed();

        choice((
            just("null").to(Value::Null).labelled("null"),
            just("true").to(Value::Boolean(true)).labelled("true"),
            just("false").to(Value::Boolean(false)).labelled("false"),
            integer.map(Value::Integer),
            string.map(|s|Value::String(Cow::Borrowed(s))),
            object.map(|entries| Value::Object(entries)),
            array.map(|members| Value::Array(members)),
        ))
        .recover_with(skip_then_retry_until(
            any().ignored(),
            one_of("]}").ignored(),
        ))
        .padded()
    })
}
