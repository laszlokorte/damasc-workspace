use damasc_lang::value::{Value, ValueBag};

#[derive(PartialEq, Eq,Debug)]
pub struct ValueId {
    id: u64,
}

#[derive(Default,Debug)]
struct IdSequence {
    next: u64,
}
impl IdSequence {
    fn next(&mut self) -> ValueId {
        let id = self.next;

        self.next += 1;

        ValueId{ id }
    }
}

#[derive(Debug)]
pub(crate) struct IdentifiedValue<'s, 'v> {
    id: ValueId,
    value: Value<'s, 'v>,
}

#[derive(Default,Debug)]
pub struct Bag<'s, 'v>{
    sequence: IdSequence,
    values: Vec<IdentifiedValue<'s, 'v>>
}

impl<'s, 'v> Bag<'s, 'v> {
    fn new() -> Self {
        Self {
            sequence: IdSequence::default(),
            values: Vec::default(),
        }
    }

    pub fn insert(&mut self, value: Value<'s,'v>) {
        self.values.push(IdentifiedValue {
            id: self.sequence.next(),
            value,
        })
    }

    pub fn remove(&mut self, value_id: ValueId) {
        self.values.retain(|v| {
            v.id != value_id
        })
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<'s,'v> From<ValueBag<'s,'v>> for Bag<'s,'v> {
    fn from(value_bag: ValueBag<'s,'v>) -> Self {
        let mut result = Self::new();
        for v in value_bag.values {
            result.insert(v);
        }
        
        result
    }
}