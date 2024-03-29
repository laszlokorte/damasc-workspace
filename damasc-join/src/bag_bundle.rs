use std::collections::HashMap;

use damasc_lang::identifier::Identifier;

use crate::bag::Bag;

#[derive(Debug, Default, Clone)]
pub struct BagBundle<'s, 'v> {
    pub bags: HashMap<Identifier<'s>, Bag<'s, 'v>>,
}
