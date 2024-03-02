use chumsky::input::MapExtra;
use damasc_lang::syntax::location::Location;




pub(crate) fn meta_to_location<'a:'s, 'b, 's, 't, E: chumsky::extra::ParserExtra<'s, &'s str>>(meta: &'t MapExtra<'a, 'b, &'s str, E>) -> Location {
    let span = meta.span();

    Location::new(span.start, span.end)
}