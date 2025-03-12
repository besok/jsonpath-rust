use crate::parser::model::Filter;
use crate::query::queryable::Queryable;
use crate::query::state::{Data, Pointer, State};
use crate::query::Query;

impl Query for Filter {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        let root = state.root;
        state.flat_map(|p| {
            if p.is_internal() {
                Data::Value(self.filter_item(p, root).into())
            } else if let Some(items) = p.inner.as_array() {
                Data::Refs(
                    items
                        .into_iter()
                        .enumerate()
                        .filter(|(_, item)| self.filter_item(Pointer::empty(*item), root))
                        .map(|(idx, item)| Pointer::idx(item, p.path.clone(), idx))
                        .collect(),
                )
            } else if let Some(items) = p.inner.as_object() {
                Data::Refs(
                    items
                        .into_iter()
                        .filter(|(_, item)| self.filter_item(Pointer::empty(*item), root))
                        .map(|(key, item)| Pointer::key(item, p.path.clone(), key))
                        .collect(),
                )
            } else {
                return Data::Nothing;
            }
        })
    }
}

impl Filter {
    fn process_elem<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        let process_cond = |filter: &Filter| {
            filter
                .process(state.clone())
                .ok_val()
                .and_then(|v| v.as_bool())
                .unwrap_or_default()
        };
        match self {
            Filter::Or(ors) => State::bool(ors.iter().any(process_cond), state.root),
            Filter::And(ands) => State::bool(ands.iter().all(process_cond), state.root),
            Filter::Atom(atom) => atom.process(state),
        }
    }

    fn filter_item<'a, T: Queryable>(&self, item: Pointer<'a, T>, root: &T) -> bool {

        self.process_elem(State::data(root, Data::Ref(item.clone())))
            .ok_val()
            .and_then(|v| v.as_bool())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::query::{js_path, QueryRes};
    use serde_json::json;

    #[test]
    fn smoke_ok() {
        let json = json!({"a" : [1,2,3]});

        assert_eq!(
            js_path("$.a[? @ > 1]", &json),
            Ok(vec![
                QueryRes::Ref(&json!(2), "$.['a'][1]".to_string()),
                QueryRes::Ref(&json!(3), "$.['a'][2]".to_string()),
            ])
        );
    }

    #[test]
    fn existence() {
        let json = json!({
          "a": {
            "a":{"b":1},
            "c": {
              "b": 2
            },
            "d": {
              "b1": 3
            }
          }
        });
        assert_eq!(
            js_path("$.a[?@.b]", &json),
            Ok(vec![
                (&json!({"b":1}), "$.['a'].['a']".to_string()).into(),
                (&json!({"b":2}), "$.['a'].['c']".to_string()).into(),
            ])
        );
    }

    #[test]
    fn existence_or() {
        let json = json!({
          "a": {
            "a":{"b":1},
            "c": {
              "b": 2
            },
            "d": {
              "b1": 3
            },
            "e": {
              "b2": 3
            }
          }
        });
        assert_eq!(
            js_path("$.a[?@.b || @.b1]", &json),
            Ok(vec![
                (&json!({"b":1}), "$.['a'].['a']".to_string()).into(),
                (&json!({"b":2}), "$.['a'].['c']".to_string()).into(),
                (&json!({"b1":3}), "$.['a'].['d']".to_string()).into(),
            ])
        );
    }
}
