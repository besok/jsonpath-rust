use crate::parser::model::{Comparable, Comparison, Literal, SingularQuery, SingularQuerySegment};
use crate::query::queryable::Queryable;
use crate::query::state::{Data, Pointer, State};
use crate::query::Query;

impl Query for Comparison {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        let root = state.root;
        let (lhs, rhs) = self.vals();
        let lhs = lhs.process(state.clone());
        let rhs = rhs.process(state);
        match self {
            Comparison::Eq(..) => State::bool(eq(lhs, rhs), root),
            Comparison::Ne(..) => State::bool(!eq(lhs, rhs), root),
            Comparison::Gt(..) => State::bool(lt(rhs, lhs), root),
            Comparison::Gte(..) => State::bool(lt(rhs.clone(), lhs.clone()) || eq(lhs, rhs), root),
            Comparison::Lt(..) => State::bool(lt(lhs, rhs), root),
            Comparison::Lte(..) => State::bool(lt(lhs.clone(), rhs.clone()) || eq(lhs, rhs), root),
        }
    }
}

fn lt<'a, T: Queryable>(lhs: State<'a, T>, rhs: State<'a, T>) -> bool {
    let cmp = |lhs: &T, rhs: &T| {
        let lhs_f64 = lhs.as_f64().or_else(|| lhs.as_i64().map(|v| v as f64));
        let rhs_f64 = rhs.as_f64().or_else(|| rhs.as_i64().map(|v| v as f64));
        if let (Some(lhs_num), Some(rhs_num)) = (lhs_f64, rhs_f64) {
            lhs_num < rhs_num
        } else if let (Some(lhs), Some(rhs)) = (lhs.as_str(), rhs.as_str()) {
            lhs < rhs
        } else {
            false
        }
    };

    match (lhs.data, rhs.data) {
        (Data::Value(lhs), Data::Value(rhs)) => cmp(&lhs, &rhs),
        (Data::Value(v), Data::Ref(p)) => cmp(&v, p.inner),
        (Data::Ref(p), Data::Value(v)) => cmp(p.inner, &v),
        (Data::Ref(lhs), Data::Ref(rhs)) => cmp(lhs.inner, rhs.inner),
        _ => false,
    }
}

fn eq<'a, T: Queryable>(lhs_state: State<'a, T>, rhs_state: State<'a, T>) -> bool {
    match (lhs_state.data, rhs_state.data) {
        (Data::Value(lhs), Data::Value(rhs)) => eq_json(&lhs, &rhs),
        (Data::Value(v), Data::Ref(p)) => eq_json(&v, p.inner),
        (Data::Ref(p), Data::Value(v)) => eq_json(&v, p.inner),
        (Data::Ref(lhs), Data::Ref(rhs)) => eq_json(lhs.inner, rhs.inner),
        (Data::Refs(lhs), Data::Refs(rhs)) => lhs == rhs,
        (Data::Ref(r), Data::Refs(rhs)) => eq_ref_to_array(r, &rhs),
        (Data::Nothing, Data::Nothing) => true,
        _ => false,
    }
}
/// Compare two JSON values for equality.
/// For numbers, it should implement interoperability for integer and float
fn eq_json<T: Queryable>(lhs: &T, rhs: &T) -> bool {
    let lhs_f64 = lhs.as_f64().or_else(|| lhs.as_i64().map(|v| v as f64));
    let rhs_f64 = rhs.as_f64().or_else(|| rhs.as_i64().map(|v| v as f64));

    if let (Some(lhs_num), Some(rhs_num)) = (lhs_f64, rhs_f64) {
        (lhs_num - rhs_num).abs() < f64::EPSILON
    } else {
        lhs == rhs
    }
}
fn eq_ref_to_array<T: Queryable>(r: Pointer<T>, rhs: &Vec<Pointer<T>>) -> bool {
    r.inner.as_array().map_or(false, |array| {
        eq_arrays(array, &rhs.iter().map(|p| p.inner).collect::<Vec<_>>())
    })
}

fn eq_arrays<T: Queryable>(lhs: &Vec<T>, rhs: &Vec<&T>) -> bool {
    lhs.len() == rhs.len() && lhs.iter().zip(rhs.iter()).all(|(a, b)| eq_json(a, *b))
}

#[cfg(test)]
mod tests {
    use crate::parser::model::{
        Comparable, Comparison, Literal, SingularQuery, SingularQuerySegment,
    };
    use crate::query::state::{Data, Pointer, State};
    use crate::query::Query;
    use crate::singular_query;
    use crate::{cmp, comparable, lit, q_segment, q_segments};
    use serde_json::json;
    #[test]
    fn eq_comp_val() {
        let data = json!({"key": "value"});
        let state = State::root(&data);

        let comparison = Comparison::Eq(comparable!(lit!(s "key")), comparable!(lit!(s "key")));
        let result = comparison.process(state);
        assert_eq!(result.ok_val(), Some(json!(true)));
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
        assert_eq!(result.ok_val(), Some(json!(true)));
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
        assert_eq!(result.ok_val(), Some(json!(true)));
    }

    #[test]
    fn neq_comp_val() {
        let data = json!({"key": "value"});
        let state = State::root(&data);

        let comparison = Comparison::Ne(comparable!(lit!(s "key")), comparable!(lit!(s "key")));
        let result = comparison.process(state);
        assert_eq!(result.ok_val(), Some(json!(false)));
    }

    #[test]
    fn less_than() {
        let data = json!({"key": 3});
        let state = State::root(&data);

        let comparison = Comparison::Lt(
            comparable!(lit!(i 2)),
            comparable!(> singular_query!(@ key)),
        );
        let result = comparison.process(state);
        assert_eq!(result.ok_val(), Some(json!(true)));
    }

    #[test]
    fn less_than_false() {
        let data = json!({"key": 1});
        let state = State::root(&data);

        let comparison = Comparison::Lt(
            comparable!(lit!(i 2)),
            comparable!(> singular_query!(@ key)),
        );
        let result = comparison.process(state);
        assert_eq!(result.ok_val(), Some(json!(false)));
    }

    #[test]
    fn less_true() {
        let data = json!({"key": 1});
        let state = State::root(&data);

        let comparison = Comparison::Lt(
            comparable!(> singular_query!(@ key)),
            comparable!(lit!(i 2)),
        );
        let result = comparison.process(state);
        assert_eq!(result.ok_val(), Some(json!(true)));
    }
}
