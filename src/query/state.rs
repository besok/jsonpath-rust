use crate::query::queryable::Queryable;
use crate::query::QueryPath;
use std::fmt::{Display, Formatter};

/// Represents the state of a query, including the current data and the root object.
/// It is used to track the progress of a query as it traverses through the data structure.
#[derive(Debug, Clone, PartialEq)]
pub struct State<'a, T: Queryable> {
    pub data: Data<'a, T>,
    pub root: &'a T,
}

impl<'a, T: Queryable> Display for State<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl<'a, T: Queryable> From<&'a T> for State<'a, T> {
    fn from(root: &'a T) -> Self {
        State::root(root)
    }
}

impl<'a, T: Queryable> State<'a, T> {
    pub fn bool(b: bool, root: &T) -> State<T> {
        State::data(root, Data::Value(b.into()))
    }

    pub fn i64(i: i64, root: &T) -> State<T> {
        State::data(root, Data::Value(i.into()))
    }
    pub fn str(v: &str, root: &'a T) -> State<'a, T> {
        State::data(root, Data::Value(v.into()))
    }

    pub fn shift_to_root(self) -> State<'a, T> {
        State::root(self.root)
    }

    pub fn root(root: &'a T) -> Self {
        State {
            root,
            data: Data::new_ref(Pointer::new(root, "$".to_string())),
        }
    }

    pub fn nothing(root: &'a T) -> Self {
        State {
            root,
            data: Data::Nothing,
        }
    }

    pub fn data(root: &'a T, data: Data<'a, T>) -> Self {
        State { root, data }
    }

    pub fn ok_ref(self) -> Option<Vec<Pointer<'a, T>>> {
        self.data.ok_ref()
    }

    pub fn ok_val(self) -> Option<T> {
        match self.data {
            Data::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn is_nothing(&self) -> bool {
        matches!(&self.data, Data::Nothing)
    }

    pub fn reduce(self, other: State<'a, T>) -> State<'a, T> {
        State {
            root: self.root,
            data: self.data.reduce(other.data),
        }
    }
    pub fn flat_map<F>(self, f: F) -> State<'a, T>
    where
        F: Fn(Pointer<'a, T>) -> Data<'a, T>,
    {
        State {
            root: self.root,
            data: self.data.flat_map(f),
        }
    }
}

/// Represents the data that is being processed in the query.
/// It can be a reference to a single object, a collection of references,
#[derive(Debug, Clone, PartialEq)]
pub enum Data<'a, T: Queryable> {
    Ref(Pointer<'a, T>),
    Refs(Vec<Pointer<'a, T>>),
    Value(T),
    Nothing,
}

impl<'a, T: Queryable> Display for Data<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Ref(p) => write!(f, "&{}", p),
            Data::Refs(p) => write!(
                f,
                "{}",
                p.iter()
                    .map(|ptr| ptr.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            Data::Value(v) => write!(f, "{:?}", v),
            Data::Nothing => write!(f, "Nothing"),
        }
    }
}

impl<'a, T: Queryable> Default for Data<'a, T> {
    fn default() -> Self {
        Data::Nothing
    }
}

impl<'a, T: Queryable> Data<'a, T> {
    pub fn reduce(self, other: Data<'a, T>) -> Data<'a, T> {
        match (self, other) {
            (Data::Ref(data), Data::Ref(data2)) => Data::Refs(vec![data, data2]),
            (Data::Ref(data), Data::Refs(data_vec)) => {
                Data::Refs(vec![data].into_iter().chain(data_vec).collect())
            }
            (Data::Refs(data_vec), Data::Ref(data)) => {
                Data::Refs(data_vec.into_iter().chain(vec![data]).collect())
            }
            (Data::Refs(data_vec), Data::Refs(data_vec2)) => {
                Data::Refs(data_vec.into_iter().chain(data_vec2).collect())
            }
            (d @ (Data::Ref(_) | Data::Refs(..)), Data::Nothing) => d,
            (Data::Nothing, d @ (Data::Ref(_) | Data::Refs(..))) => d,
            _ => Data::Nothing,
        }
    }

    pub fn flat_map<F>(self, f: F) -> Data<'a, T>
    where
        F: Fn(Pointer<'a, T>) -> Data<'a, T>,
    {
        match self {
            Data::Ref(data) => f(data),
            Data::Refs(data_vec) => Data::Refs(
                data_vec
                    .into_iter()
                    .flat_map(|data| match f(data) {
                        Data::Ref(data) => vec![data],
                        Data::Refs(data_vec) => data_vec,
                        _ => vec![],
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => Data::Nothing,
        }
    }

    /// Returns the inner value if it is a single reference.
    /// If it is a collection of references, it returns the first one.
    /// If it is a value, it returns None.
    pub fn ok_ref(self) -> Option<Vec<Pointer<'a, T>>> {
        match self {
            Data::Ref(data) => Some(vec![data]),
            Data::Refs(data) => Some(data),
            _ => None,
        }
    }

    /// Returns the inner value if it is a single value.
    /// If it is a reference or a collection of references, it returns None.
    pub fn ok_val(self) -> Option<T> {
        match self {
            Data::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn new_ref(data: Pointer<'a, T>) -> Data<'a, T> {
        Data::Ref(data)
    }

    pub fn new_refs(data: Vec<Pointer<'a, T>>) -> Data<'a, T> {
        Data::Refs(data)
    }
}

/// Represents a pointer to a specific location in the data structure.
/// It contains a reference to the data and a path that indicates the location of the data in the structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Pointer<'a, T: Queryable> {
    pub inner: &'a T,
    pub path: QueryPath,
}

impl<'a, T: Queryable> Display for Pointer<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}='{}'", self.inner, self.path)
    }
}

impl<'a, T: Queryable> Pointer<'a, T> {
    pub fn new(inner: &'a T, path: QueryPath) -> Self {
        Pointer { inner, path }
    }

    pub fn key(inner: &'a T, path: QueryPath, key: &str) -> Self {
        Pointer {
            inner,
            path: format!("{}.['{}']", path, key),
        }
    }
    pub fn idx(inner: &'a T, path: QueryPath, index: usize) -> Self {
        Pointer {
            inner,
            path: format!("{}[{}]", path, index),
        }
    }

    pub fn empty(inner: &'a T) -> Self {
        Pointer {
            inner,
            path: String::new(),
        }
    }

    pub fn is_internal(&self) -> bool {
        self.path.is_empty()
    }
}
