use crate::parser::model2::{Comparable, Comparison, Literal, SingularQuery, SingularQuerySegment};
use crate::query::queryable::Queryable;
use crate::query::selector::{process_index, process_key};
use crate::query::state::{Data, Pointer, State};
use crate::query::Query;

impl Query for Comparison {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        let root = state.root;
        match self {
            Comparison::Eq(lhs, rhs) => {
                let lhs = lhs.process(state.clone());
                let rhs = rhs.process(state);
                bool_to_state(eq(lhs, rhs), root)
            }
            Comparison::Ne(lhs, rhs) => {
                let lhs = lhs.process(state.clone());
                let rhs = rhs.process(state);
                bool_to_state(!eq(lhs, rhs), root)
            }
            Comparison::Gt(lhs, rhs) => State::nothing(state.root),
            Comparison::Gte(lhs, rhs) => State::nothing(state.root),
            Comparison::Lt(lhs, rhs) => State::nothing(state.root),
            Comparison::Lte(lhs, rhs) => State::nothing(state.root),
        }
    }
}

fn bool_to_state<T: Queryable>(b: bool, root: &T) -> State<T> {
    State::data(root, Data::Value(b.into()))
}

fn eq<'a, T: Queryable>(lhs_state: State<'a, T>, rhs_state: State<'a, T>) -> bool {
    match (lhs_state.data, rhs_state.data) {
        (Data::Value(lhs), Data::Value(rhs)) => lhs == rhs,
        (Data::Value(v), Data::Ref(p)) | (Data::Ref(p), Data::Value(v)) => v == *p.inner,
        (Data::Ref(lhs), Data::Ref(rhs)) => lhs.inner == rhs.inner,
        (Data::Refs(lhs), Data::Refs(rhs)) => lhs == rhs,
        (Data::Ref(r), Data::Refs(rhs)) => eq_ref_to_array(r, &rhs),

        _ => false,
    }
}

fn eq_ref_to_array<T: Queryable>(r: Pointer<T>, rhs: &Vec<Pointer<T>>) -> bool {
    r.inner.as_array().map_or(false, |array| {
        eq_arrays(array, &rhs.iter().map(|p| p.inner).collect::<Vec<_>>())
    })
}

fn eq_arrays<T: PartialEq>(lhs: &Vec<T>, rhs: &Vec<&T>) -> bool {
    lhs.len() == rhs.len() && lhs.iter().zip(rhs.iter()).all(|(a, b)| a == *b)
}

#[cfg(test)]
mod tests {
    use crate::parser::model2::{
        Comparable, Comparison, Literal, SingularQuery, SingularQuerySegment,
    };
    use crate::q_segments;
    use crate::query::state::{Data, Pointer, State};
    use crate::query::Query;
    use crate::singular_query;
    use crate::{cmp, comparable, lit, q_segment};
    use serde_json::json;

    #[test]
    fn eq_comp_val() {
        let data = json!({"key": "value"});
        let state = State::root(&data);

        let comparison = Comparison::Eq(comparable!(lit!(s "key")), comparable!(lit!(s "key")));
        let result = comparison.process(state);
        assert_eq!(result.val(), Some(json!(true)));
    }

    #[test]
    fn eq_comp_ref() {
        let data = json!({"key": "value"});
        let state = State::root(&data);

        let comparison = Comparison::Eq(
            comparable!(lit!(s "value")),
            comparable!(> singular_query!(@ key)),
        );

        let result = comparison.process(state);
        assert_eq!(result.val(), Some(json!(true)));
    }

    #[test]
    fn eq_comp_queries() {
        let data = json!({"key": "value", "key2": "value"});
        let state = State::root(&data);

        let comparison = Comparison::Eq(
            comparable!(> singular_query!(@ key)),
            comparable!(> singular_query!(key2)),
        );
        let result = comparison.process(state);
        assert_eq!(result.val(), Some(json!(true)));
    }

    #[test]
    fn neq_comp_val() {
        let data = json!({"key": "value"});
        let state = State::root(&data);

        let comparison = Comparison::Ne(comparable!(lit!(s "key")), comparable!(lit!(s "key")));
        let result = comparison.process(state);
        assert_eq!(result.val(), Some(json!(false)));
    }
}
