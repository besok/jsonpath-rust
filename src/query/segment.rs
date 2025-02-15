use crate::parser::model2::{Segment, Selector};
use crate::query::{Data, Step, Query};
use crate::query::queryable::Queryable;

impl Query for Segment {
    fn process<'a, T: Queryable>(&self, progress: Step<'a, T>) -> Step<'a, T> {
        match self {
            Segment::Descendant => {unimplemented!()}
            Segment::Selector(selector) => {
                match selector {
                    Selector::Name(key) => {
                        progress.flat_map(|Data { pointer, path }| {
                            pointer.get(key)
                                .map(|v| Step::new_ref(Data::new_key(v, path, key)))
                                .unwrap_or_default()
                        })
                    }
                    Selector::Wildcard => {unimplemented!()}
                    Selector::Index(_) => {unimplemented!()}
                    Selector::Slice(_, _, _) => {unimplemented!()}
                    Selector::Filter(_) => {unimplemented!()}
                }
            }
            Segment::Selectors(_) => {unimplemented!()}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_process() {
        let value = json!({"key": "value"});
        let segment = Segment::Selector(Selector::Name("key".to_string()));
        let step = segment.process(Step::new_ref(Data::new(&value, "$".to_string())));

        assert_eq!(step.ok(), Some(Data::new(&json!("value"), "$.['key']".to_string())));
    }
}