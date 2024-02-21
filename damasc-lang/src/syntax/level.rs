use core::hash::Hash;
use std::fmt::Debug;
use crate::literal::Literal;

use crate::identifier::Identifier;

pub trait SyntaxLevel: Clone + Debug + Hash + Eq + PartialEq + Ord + PartialOrd {
	type Annotation : Clone + Hash + Debug + Eq + PartialEq + Ord + PartialOrd;
	type Identifier<'s> : Clone + Hash + Debug + Eq + PartialEq + Ord + PartialOrd;
	type Literal<'s> : Clone + Hash + Debug + Eq + PartialEq + Ord + PartialOrd;
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct EmptyLevel {}

impl SyntaxLevel for EmptyLevel {
	type Annotation = ();
	type Identifier<'s> = Identifier<'s>;
	type Literal<'s> = Literal<'s>;
}