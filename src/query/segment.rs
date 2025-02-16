use crate::parser::model2::{Segment, Selector};
use crate::query::queryable::Queryable;
use crate::query::{Data, Query, Step};

impl Query for Segment {
    fn process<'a, T: Queryable>(&self, step: Step<'a, T>) -> Step<'a, T> {
        match self {
            Segment::Descendant => step.flat_map(process_descendant),
            Segment::Selector(selector) => selector.process(step),
            Segment::Selectors(selectors) => process_selectors(step, selectors),
        }
    }
}


fn process_selectors<'a, T: Queryable>(step: Step<'a, T>, selectors: &Vec<Selector>) -> Step<'a, T> {
    selectors
        .into_iter()
        .map(|s| s.process(step.clone()))
        .reduce(Step::reduce)
        .unwrap_or_default()
}

fn process_descendant<T: Queryable>(data: Data<T>) -> Step<T> {
    if let Some(array) = data.pointer.as_array() {
        Step::new_refs(
            array
                .iter()
                .enumerate()
                .map(|(i, elem)| Data::idx(elem, data.path.clone(), i))
                .collect(),
        ).reduce(Step::Ref(data))

    } else if let Some(object) = data.pointer.as_object() {
        Step::new_refs(
            object
                .into_iter()
                .map(|(key, value)| Data::key(value, data.path.clone(), key))
                .collect(),
        ).reduce(Step::Ref(data))
    } else {
        Step::Nothing
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::parser::model2::{Segment, Selector};
    use crate::query::{Data, Query, Step};

    #[test]
    fn test_process_selectors() {
        let value = json!({"firstName": "John", "lastName" : "doe",});
        let segment = Segment::Selectors(vec![
            Selector::Name("firstName".to_string()),
            Selector::Name("lastName".to_string()),
        ]);
        let step = segment.process(Step::new_ref(Data::new(&value, "$".to_string())));

        assert_eq!(
            step.ok(),
            Some(vec![
                Data::new(&json!("John"), "$.['firstName']".to_string()),
                Data::new(&json!("doe"), "$.['lastName']".to_string())
            ])
        );
    }

    #[test]
    fn test_process_descendant() {
        let value = json!([{"name": "John"}, {"name": "doe"}]);
        let segment = Segment::Descendant;
        let step = segment.process(Step::new_ref(Data::new(&value, "$".to_string())));

        assert_eq!(
            step.ok(),
            Some(vec![
                Data::new(&json!({"name": "John"}), "$[0]".to_string()),
                Data::new(&json!({"name": "doe"}), "$[1]".to_string()),
                Data::new(&json!([{"name": "John"}, {"name": "doe"}]), "$".to_string()),

            ])
        );
    }



}