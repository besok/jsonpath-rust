use crate::parser2::model2::{FnArg, TestFunction};
use crate::query::queryable::Queryable;
use crate::query::state::{Data, Pointer, State};
use crate::query::Query;
use regex::Regex;
use std::borrow::Cow;

impl TestFunction {
    pub fn apply<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        match self {
            TestFunction::Length(arg) => length(arg.process(state)),
            TestFunction::Count(arg) => count(arg.process(state)),
            TestFunction::Match(lhs, rhs) => {
                regex(lhs.process(state.clone()), rhs.process(state), false)
            }
            TestFunction::Search(lhs, rhs) => {
                regex(lhs.process(state.clone()), rhs.process(state), true)
            }
            TestFunction::Custom(name, args) => custom(name, args, state),
            TestFunction::Value(arg) => value(arg.process(state)),
            _ => State::nothing(state.root),
        }
    }
}

impl Query for FnArg {
    fn process<'a, T: Queryable>(&self, step: State<'a, T>) -> State<'a, T> {
        match self {
            FnArg::Literal(lit) => lit.process(step),
            FnArg::Test(test) => test.process(step),
            FnArg::Filter(filter) => filter.process(step),
        }
    }
}

impl Query for TestFunction {
    fn process<'a, T: Queryable>(&self, step: State<'a, T>) -> State<'a, T> {
        self.apply(step)
    }
}

fn custom<'a, T: Queryable>(name: &str, args: &Vec<FnArg>, state: State<'a, T>) -> State<'a, T> {
    let args = args
        .into_iter()
        .map(|v| v.process(state.clone()))
        .flat_map(|v| match v.data {
            Data::Value(v) => vec![Cow::Owned(v)],
            Data::Ref(Pointer { inner, .. }) => vec![Cow::Borrowed(inner)],
            Data::Refs(v) => v.into_iter().map(|v| Cow::Borrowed(v.inner)).collect(),
            _ => vec![],
        })
        .collect::<Vec<_>>();

    State::data(
        state.root,
        Data::Value(Queryable::extension_custom(name, args)),
    )
}

/// Returns the length/size of the object.
///
/// # Returns
///
/// Returns a `Progress` enum containing either:
/// - `Progress::Data` with a vector of references to self and the query path for strings/arrays/objects
/// - `Progress::Nothing` for other types
///
/// The returned length follows JSON path length() function semantics based on the type:
/// - String type: Number of Unicode scalar values
/// - Array type: Number of elements
/// - Object type: Number of members
/// - Other types: Nothing
fn length<T: Queryable>(state: State<T>) -> State<T> {
    let from_item = |item: &T| {
        if let Some(v) = item.as_str() {
            State::i64(v.chars().count() as i64, state.root)
        } else if let Some(items) = item.as_array() {
            State::i64(items.len() as i64, state.root)
        } else if let Some(items) = item.as_object() {
            State::i64(items.len() as i64, state.root)
        } else {
            State::nothing(state.root)
        }
    };

    match state.data {
        Data::Ref(Pointer { inner, .. }) => from_item(inner),
        Data::Refs(items) => State::i64(items.len() as i64, state.root),
        Data::Value(item) => from_item(&item),
        Data::Nothing => State::nothing(state.root),
    }
}

/// The count() function extension provides a way
/// to obtain the number of nodes in a nodelist
/// and make that available for further processing in the filter expression
fn count<T: Queryable>(state: State<T>) -> State<T> {
    let to_state = |count: i64| State::i64(count, state.root);

    match state.data {
        Data::Ref(..) | Data::Value(..) => to_state(1),
        Data::Refs(items) => to_state(items.len() as i64),
        Data::Nothing => State::nothing(state.root),
    }
}
/// The match() function extension provides
/// a way to check whether (the entirety of; see Section 2.4.7)
/// a given string matches a given regular expression,
/// which is in the form described in [RFC9485].
///
/// Its arguments are instances of ValueType
/// (possibly taken from a singular query,
/// as for the first argument in the example above).
/// If the first argument is not a string
/// or the second argument is not a string conforming to [RFC9485],
/// the result is LogicalFalse. Otherwise, the string that is the first argument is matched against
/// the I-Regexp contained in the string that is the second argument; the result is LogicalTrue
/// if the string matches the I-Regexp and is LogicalFalse otherwise.
fn regex<'a, T: Queryable>(lhs: State<'a, T>, rhs: State<'a, T>, substr: bool) -> State<'a, T> {
    let to_state = |b| State::bool(b, lhs.root);
    let regex = |v: &str, r: Regex| {
        if substr {
            r.find(v).is_some()
        } else {
            r.is_match(v)
        }
    };
    let to_str = |s: State<'a, T>| match s.data {
        Data::Value(v) => v.as_str().map(|s| s.to_string()),
        Data::Ref(Pointer { inner, .. }) => inner.as_str().map(|s| s.to_string()),
        _ => None,
    };

    match (to_str(lhs), to_str(rhs)) {
        (Some(lhs), Some(rhs)) => Regex::new(rhs.trim_matches(|c| c == '\'' || c == '"'))
            .map(|re| to_state(regex(&lhs, re)))
            .unwrap_or(to_state(false)),
        _ => to_state(false),
    }
}

fn value<T: Queryable>(state: State<T>) -> State<T> {
    match state.data {
        Data::Ref(..) | Data::Value(..) => state,
        Data::Refs(items) if items.len() == 1 => {
            State::data(state.root, Data::Ref(items[0].clone()))
        }
        _ => State::nothing(state.root),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser2::model2::Segment;
    use crate::parser2::model2::Selector;
    use crate::parser2::model2::Test;
    use crate::parser2::model2::TestFunction;
    use crate::query::state::{Data, Pointer, State};
    use crate::query::test_function::{regex, FnArg};
    use crate::query::Query;
    use crate::{arg, q_segment, segment, selector, test, test_fn};
    use serde_json::json;

    #[test]
    fn test_len() {
        let json = json!({"array": [1,2,3]});
        let state = State::root(&json);

        let query = test_fn!(length arg!(t test!(@ segment!(selector!(array)))));
        let res = query.process(state);

        assert_eq!(res.ok_val(), Some(json!(3)));
    }

    #[test]
    fn test_match_1() {
        let json = json!({"a": "abc sdgfudsf","b": "abc.*"});
        let state = State::root(&json);

        let query = test_fn!(match
            arg!(t test!(@ segment!(selector!(a)))),
            arg!(t test!(@ segment!(selector!(b))))
        );
        let res = query.process(state);

        assert_eq!(res.ok_val(), Some(json!(true)));
    }

    #[test]
    fn test_count_1() {
        let json = json!({"array": [1,2,3]});
        let state = State::root(&json);

        let query = test_fn!(count arg!(t test!(@ segment!(selector!(array)))));
        let res = query.process(state);

        assert_eq!(res.ok_val(), Some(json!(1)));
    }

    #[test]
    fn test_search() {
        let json = json!("123");
        let state = State::root(&json);
        let reg = State::str("[a-z]+",&json,);

        let res = regex(state, reg, true);

        assert_eq!(res.ok_val(), Some(json!(false)));
    }
}
