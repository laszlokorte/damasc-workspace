use std::borrow::Cow;

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier<'a> {
    pub name: Cow<'a, str>,
}

impl Identifier<'_> {
    pub(crate) fn deep_clone<'y>(&self) -> Identifier<'y> {
        Identifier {
            name: Cow::Owned(self.name.as_ref().into()),
        }
    }
}

impl std::fmt::Display for Identifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
