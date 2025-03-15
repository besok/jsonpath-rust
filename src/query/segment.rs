use crate::parser::model::{Segment, Selector};
use crate::query::queryable::Queryable;
use crate::query::state::{Data, Pointer, State};
use crate::query::Query;

impl Query for Segment {
    fn process<'a, T: Queryable>(&self, step: State<'a, T>) -> State<'a, T> {
        match self {
            Segment::Descendant(segment) => segment.process(step.flat_map(process_descendant)),
            Segment::Selector(selector) => selector.process(step),
            Segment::Selectors(selectors) => process_selectors(step, selectors),
        }
    }
}

fn process_selectors<'a, T: Queryable>(
    step: State<'a, T>,
    selectors: &Vec<Selector>,
) -> State<'a, T> {
    selectors
        .into_iter()
        .map(|s| s.process(step.clone()))
        .reduce(State::reduce)
        .unwrap_or(step.root.into())
}

fn process_descendant<T: Queryable>(data: Pointer<T>) -> Data<T> {
    if let Some(array) = data.inner.as_array() {
        Data::Ref(data.clone()).reduce(
            Data::new_refs(
                array
                    .iter()
                    .enumerate()
                    .map(|(i, elem)| Pointer::idx(elem, data.path.clone(), i))
                    .collect(),
            )
            .flat_map(process_descendant),
        )
    } else if let Some(object) = data.inner.as_object() {
        Data::Ref(data.clone()).reduce(
            Data::new_refs(
                object
                    .into_iter()
                    .map(|(key, value)| Pointer::key(value, data.path.clone(), key))
                    .collect(),
            )
            .flat_map(process_descendant),
        )
    } else {
        Data::Nothing
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::model::{Segment, Selector};
    use crate::query::state::{Pointer, State};
    use crate::query::Query;
    use serde_json::json;

    #[test]
    fn test_process_selectors() {
        let value = json!({"firstName": "John", "lastName" : "doe",});
        let segment = Segment::Selectors(vec![
            Selector::Name("firstName".to_string()),
            Selector::Name("lastName".to_string()),
        ]);
        let step = segment.process(State::root(&value));

        assert_eq!(
            step.ok_ref(),
            Some(vec![
                Pointer::new(&json!("John"), "$['firstName']".to_string()),
                Pointer::new(&json!("doe"), "$['lastName']".to_string())
            ])
        );
    }

    #[test]
    fn test_process_descendant() {
        let value = json!([{"name": "John"}, {"name": "doe"}]);
        let segment = Segment::Descendant(Box::new(Segment::Selector(Selector::Wildcard)));
        let step = segment.process(State::root(&value));

        assert_eq!(
            step.ok_ref(),
            Some(vec![
                Pointer::new(&json!({"name": "John"}), "$[0]".to_string()),
                Pointer::new(&json!({"name": "doe"}), "$[1]".to_string()),
                Pointer::new(&json!("John"), "$[0]['name']".to_string()),
                Pointer::new(&json!("doe"), "$[1]['name']".to_string()),
            ])
        );
    }

    #[test]
    fn test_process_descendant2() {
        let value = json!({"o": [0,1,[2,3]]});
        let segment = Segment::Descendant(Box::new(Segment::Selector(Selector::Index(1))));
        let step = segment.process(State::root(&value));

        assert_eq!(
            step.ok_ref(),
            Some(vec![
                Pointer::new(&json!(1), "$['o'][1]".to_string()),
                Pointer::new(&json!(3), "$['o'][2][1]".to_string()),
            ])
        );
    }
}
