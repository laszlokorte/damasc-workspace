use crate::identifier::single_identifier;
use crate::literal::single_literal;
use crate::literal::single_string_literal;
use chumsky::error::Error;
use chumsky::extra;
use chumsky::prelude::Rich;

use damasc_lang::literal::Literal;
use std::borrow::Cow;

use chumsky::Parser;

use damasc_lang::value::Value;

use chumsky::prelude::*;

pub fn single_value<'s>() -> impl Parser<'s, &'s str, Value<'s, 's>, extra::Err<Rich<'s, char>>> {
    recursive(|value| {
        let value = value.labelled("value");

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
            )
            .labelled("array")
            .as_context()
            .boxed();

        let member = single_string_literal()
            .or(single_identifier().map(|i| i.name))
            .labelled("object_key")
            .as_context()
            .then_ignore(just(':').padded())
            .then(value.labelled("value").as_context().map(Cow::Owned)).boxed();
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
                just('}')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            )
            .labelled("object")
            .as_context()
            .boxed();

        choice((
            single_literal().try_map(move |lit, span| match lit {
                Literal::Null => Ok(Value::Null),
                Literal::String(s) => Ok(Value::String(s)),
                Literal::Number(num) => num
                    .parse()
                    .map(Value::Integer).map_err(move |_| Error::<&str>::expected_found(None, None, span)),
                Literal::Boolean(b) => Ok(Value::Boolean(b)),
                Literal::Type(t) => Ok(Value::Type(t)),
            }).boxed(),
            object.map(Value::Object),
            array.map(Value::Array),
        ))
        .recover_with(skip_then_retry_until(
            any().ignored(),
            one_of("]}").ignored(),
        ))
        .padded()
    })
}
