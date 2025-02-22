use crate::parser::model2::FilterAtom;
use crate::query::queryable::Queryable;
use crate::query::state::State;
use crate::query::Query;

impl Query for FilterAtom {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        println!("FilterAtom: {}", state);
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
                println!("self {:?}", self);
                println!("test: {}, {}", res, state);
                if res.is_nothing() {
                    new_state(*not)
                } else {
                    new_state(!*not)
                }
            }
            FilterAtom::Comparison(cmp) => cmp.process(state),
        }
    }
}

fn invert_bool<T:Queryable>(state: State<T>) -> State<T> {
    let root = state.root;
    State::bool(
        !state.ok_val().and_then(|v| v.as_bool()).unwrap_or_default(),
        root,
    )
}

#[cfg(test)]
mod tests {
    use crate::parser::model2::Comparable;
    use crate::parser::model2::Literal;
    use crate::parser::model2::SingularQuery;
    use crate::parser::model2::SingularQuerySegment;
    use crate::parser::model2::{Comparison, FilterAtom};
    use crate::query::queryable::Queryable;
    use crate::query::state::State;
    use crate::query::Query;
    use crate::{and, cmp, or, singular_query};
    use crate::{atom, comparable, lit};
    use crate::{filter, q_segment};
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
            Some(false)
        );
        assert_eq!(
            atom_and.process(state).ok_val().and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}
