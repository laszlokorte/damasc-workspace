use crate::identifier::Identifier;
use crate::runtime::env::Environment;
use crate::value_type::ValueType;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashSet;

use crate::syntax::expression::Expression;
use crate::syntax::pattern::Pattern;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Value<'s, 'v> {
    Null,
    String(Cow<'s, str>),
    Integer(i64),
    Boolean(bool),
    Array(ValueArray<'s, 'v>),
    Object(ValueObjectMap<'s, 'v>),
    Type(ValueType),
    Lambda(LambdaBinding<'s, 'v>, Pattern<'s>, Expression<'s>),
}

pub(crate) type ValueArray<'s, 'v> = Vec<Cow<'v, Value<'s, 'v>>>;

pub(crate) type ValueObjectMap<'s, 'v> = BTreeMap<Cow<'s, str>, Cow<'v, Value<'s, 'v>>>;

impl<'s, 'v> Value<'s, 'v> {
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::String(..) => ValueType::String,
            Value::Integer(..) => ValueType::Integer,
            Value::Boolean(..) => ValueType::Boolean,
            Value::Array(..) => ValueType::Array,
            Value::Object(..) => ValueType::Object,
            Value::Type(..) => ValueType::Type,
            Value::Lambda(..) => ValueType::Lambda,
        }
    }

    pub fn convert(&self, specified_type: ValueType) -> Option<Value<'s, 'v>> {
        if self.get_type() == specified_type {
            return Some(self.clone());
        }

        Some(match (&self, specified_type) {
            (Value::Null, ValueType::String) => Value::String(Cow::Borrowed("null")),
            (Value::Null, ValueType::Integer) => Value::Integer(0),
            (Value::Null, ValueType::Boolean) => Value::Boolean(false),
            (Value::Null, ValueType::Array) => Value::Array(vec![]),
            (Value::Null, ValueType::Object) => Value::Object(BTreeMap::new()),
            (_, ValueType::Type) => Value::Type(self.get_type()),
            (Value::Type(t), ValueType::String) => Value::String(Cow::Owned(format!("{t}"))),
            (Value::Object(o), ValueType::Array) => Value::Array(o.values().cloned().collect()),
            (Value::Object(o), ValueType::Boolean) => Value::Boolean(!o.is_empty()),
            (Value::Array(a), ValueType::Boolean) => Value::Boolean(!a.is_empty()),
            (Value::String(s), ValueType::Boolean) => Value::Boolean(!s.is_empty()),
            (Value::String(s), ValueType::Array) => Value::Array(
                s.chars()
                    .map(|c| Cow::Owned(Value::String(Cow::Owned(c.to_string()))))
                    .collect(),
            ),
            (Value::Integer(i), ValueType::String) => Value::String(Cow::Owned(i.to_string())),
            (Value::Integer(i), ValueType::Boolean) => Value::Boolean(i != &0),
            (Value::Boolean(b), ValueType::String) => Value::String(Cow::Owned(b.to_string())),
            (Value::Boolean(b), ValueType::Integer) => Value::Integer(if *b { 1 } else { 0 }),
            (Value::Array(a), ValueType::Integer) => Value::Integer(a.len() as i64),
            (Value::Object(o), ValueType::Integer) => Value::Integer(o.len() as i64),
            _ => return None,
        })
    }

    pub fn deep_clone<'x, 'y>(&self) -> Value<'x, 'y> {
        match self {
            Value::Null => Value::Null,
            Value::String(s) => Value::String(Cow::Owned(s.to_string())),
            Value::Integer(i) => Value::Integer(*i),
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Array(a) => Value::Array(a.iter().map(|v| Cow::Owned(v.deep_clone())).collect()),
            Value::Object(o) => Value::Object(
                o.iter()
                    .map(|(k, v)| (Cow::Owned(k.to_string()), Cow::Owned(v.deep_clone())))
                    .collect(),
            ),
            Value::Type(t) => Value::Type(*t),
            Value::Lambda(e, p, b) => Value::Lambda(e.deep_clone(), p.deep_clone(), b.deep_clone()),
        }
    }
}

impl<'s, 'v> std::fmt::Display for Value<'s, 'v> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = match self {
            Value::Null => write!(f, "null"),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Integer(i) => write!(f, "{i}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Array(a) => {
                write!(f, "[")?;
                for v in a {
                    write!(f, "{v}, ",)?;
                }
                write!(f, "]")
            }
            Value::Object(o) => {
                let _ = write!(f, "{{");
                for (k, v) in o {
                    if k.chars().next().map_or(false, |c| c.is_alphabetic())
                        && k.chars().all(|x| x.is_alphanumeric())
                    {
                        write!(f, "{}: {v},", k)?;
                    } else {
                        write!(f, "\"{}\": {v},", k.replace('"', "\\\""))?;
                    }
                }
                write!(f, "}}")
            }
            Value::Type(t) => write!(f, "{t}"),
            Value::Lambda(env, pat, expr) => {
                write!(f, "fn ({pat}) => {expr}")?;

                if !env.bindings.is_empty() {
                    write!(f, " with ")?;

                    for (i, (k, v)) in env.bindings.iter().enumerate() {
                        if i > 0 {
                            write!(f, "; {}={v}", k)?;
                        } else {
                            write!(f, "{}={v}", k)?;
                        }
                    }
                }

                write!(f, "")
            }
        };
        write!(f, "")
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ValueBag<'s, 'v> {
    pub values: Vec<Value<'s, 'v>>,
}

impl std::fmt::Display for ValueBag<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for v in &self.values {
            writeln!(f, "{v};")?;
        }
        Ok(())
    }
}

impl<'s, 'v> ValueBag<'s, 'v> {
    pub fn new(values: Vec<Value<'s, 'v>>) -> Self {
        Self { values }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct LambdaBinding<'s, 'v> {
    pub bindings: BTreeMap<Identifier<'s>, Value<'s, 'v>>,
}

impl<'s, 'v> LambdaBinding<'s, 'v> {
    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    pub fn identifiers(&self) -> std::collections::HashSet<&Identifier> {
        self.bindings.keys().collect()
    }

    pub fn combine(&self, other: &Self) -> Option<Self> {
        let mut bindings = self.bindings.clone();

        for (id, value) in &other.bindings {
            match bindings.insert(id.clone(), value.clone()) {
                Some(ref old) => {
                    if old != value {
                        return None;
                    } else {
                        continue;
                    }
                }
                None => continue,
            }
        }

        Some(LambdaBinding { bindings })
    }

    pub(crate) fn replace<'y: 's, 'ii, 'ss, 'vv>(
        &mut self,
        identifiers: impl Iterator<Item = (Identifier<'y>, Value<'s, 'v>)>,
    ) {
        for (id, val) in identifiers {
            self.bindings.entry(id.clone()).and_modify(|old| *old = val);
        }
    }

    pub(crate) fn try_from_env<'x, 'y: 'x, 'ii, 'ss, 'vv>(
        env: &'_ Environment<'s, 's, 'v>,
        identifiers: impl Iterator<Item = &'x Identifier<'y>>,
        exceptions: impl Iterator<Item = &'x Identifier<'y>>,
    ) -> Result<LambdaBinding<'s, 'v>, &'x Identifier<'y>> {
        let mut binding = LambdaBinding::new();
        let skip = exceptions.collect::<HashSet<_>>();

        for id in identifiers {
            if skip.contains(id) {
                continue;
            }

            let Some(current_value) = env.bindings.get(id) else {
                return Err(id);
            };
            binding
                .bindings
                .insert(id.deep_clone(), current_value.clone());
        }

        Ok(binding)
    }

    pub(crate) fn deep_clone<'sx, 'vx>(&self) -> LambdaBinding<'sx, 'vx> {
        LambdaBinding {
            bindings: self
                .bindings
                .iter()
                .map(|(k, v)| (k.deep_clone(), v.deep_clone()))
                .collect(),
        }
    }
}

impl LambdaBinding<'_, '_> {
    pub const fn new() -> Self {
        Self {
            bindings: BTreeMap::new(),
        }
    }
}

impl<'s, 'v> From<LambdaBinding<'s, 'v>> for Environment<'s, 's, 'v> {
    fn from(val: LambdaBinding<'s, 'v>) -> Self {
        Environment {
            bindings: val
                .bindings
                .iter()
                .map(|(k, v)| (k.deep_clone(), v.deep_clone()))
                .collect(),
        }
    }
}

impl std::fmt::Display for LambdaBinding<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, v) in &self.bindings {
            writeln!(f, "{id} = {v};")?;
        }
        Ok(())
    }
}

impl Default for LambdaBinding<'_, '_> {
    fn default() -> Self {
        Self::new()
    }
}
