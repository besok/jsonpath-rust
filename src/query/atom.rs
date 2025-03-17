use crate::parser::model::FilterAtom;
use crate::query::queryable::Queryable;
use crate::query::state::{Data, State};
use crate::query::Query;

impl Query for FilterAtom {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        match self {
            FilterAtom::Filter { expr, not } => {
                let bool_res = expr.process(state);
                if *not {
                    invert_bool(bool_res)
                } else {
                    bool_res
                }
            }
            FilterAtom::Test { expr, not } => {
                let new_state = |b| State::bool(b, state.root);
                let res = expr.process(state.clone());
                if expr.is_res_bool() {
                    if *not {
                        invert_bool(res)
                    } else {
                        res
                    }
                } else {
                    let struct_check = |s: &T| {
                        if let Some(arr) = s.as_array() {
                            !arr.is_empty()
                        } else if let Some(obj) = s.as_object() {
                            !obj.is_empty()
                        } else if let Some(str) = s.as_str() {
                            !str.is_empty()
                        } else {
                            true
                        }
                    };

                    let struct_presented = match res.data {
                        Data::Ref(v) => struct_check(v.inner),
                        Data::Refs(e) if e.is_empty() => false,
                        Data::Refs(elems) => elems.iter().map(|v| v.inner).all(struct_check),
                        _ => false,
                    };

                    if struct_presented {
                        new_state(!*not)
                    } else {
                        new_state(*not)
                    }
                }
            }
            FilterAtom::Comparison(cmp) => cmp.process(state),
        }
    }
}

fn invert_bool<T: Queryable>(state: State<T>) -> State<T> {
    let root = state.root;
    State::bool(
        !state.ok_val().and_then(|v| v.as_bool()).unwrap_or_default(),
        root,
    )
}

#[cfg(test)]
mod tests {
    use crate::parser::model::Comparable;
    use crate::parser::model::Literal;
    use crate::parser::model::SingularQuery;
    use crate::parser::model::SingularQuerySegment;
    use crate::parser::model::{Comparison, FilterAtom};
    use crate::q_segment;
    use crate::query::queryable::Queryable;
    use crate::query::state::State;
    use crate::query::Query;
    use crate::{atom, comparable, lit};
    use crate::{cmp, singular_query};
    use crate::{filter_, q_segments};
    use serde_json::json;

    #[test]
    fn test_comparison() {
        let json = json!({"i": 1});
        let atom = atom!(comparable!(lit!(i 1)), ">=", comparable!(lit!(i 1)));
        let state = State::root(&json);
        let res = atom.process(state);
        assert_eq!(res.ok_val().and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn test_not_filter_atom() {
        let json = json!({"a": 1 , "b": 2});
        let state = State::root(&json);

        let f1 = filter_!(atom!(
            comparable!(> SingularQuery::Current(vec![])),
            ">",
            comparable!(lit!(i 2))
        ));
        let f2 = filter_!(atom!(
            comparable!(> singular_query!(b)),
            "!=",
            comparable!(> singular_query!(a))
        ));

        let atom_or = atom!(!filter_!(or f1.clone(), f2.clone()));
        let atom_and = atom!(!filter_!(and f1, f2));

        assert_eq!(
            atom_or
                .process(state.clone())
                .ok_val()
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            atom_and.process(state).ok_val().and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}
