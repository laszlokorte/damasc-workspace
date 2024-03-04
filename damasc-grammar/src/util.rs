use chumsky::input::MapExtra;
use damasc_lang::syntax::location::Location;

pub(crate) fn meta_to_location<'a: 'e, 'b, 's, 't, 'e:'s, E: chumsky::extra::ParserExtra<'e, &'s str>>(
    meta: &'t MapExtra<'a, 'b, &'s str, E>,
) -> Location {
    let span = meta.span();

    Location::new(span.start, span.end)
}
