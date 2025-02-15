use crate::parser::model2::{Segment, Selector};
use crate::query::queryable::Queryable;
use crate::query::{Data, Query, QueryPath, Step};

impl Segment {
    fn process_key<'a, T: Queryable>(
        &self,
        Data { pointer, path }: Data<'a, T>,
        key: &str,
    ) -> Step<'a, T> {
        pointer
            .get(key)
            .map(|v| Step::new_ref(Data::new_key(v, path, key)))
            .unwrap_or_default()
    }

    fn process_index<'a, T: Queryable>(
        &self,
        Data { pointer, path }: Data<'a, T>,
        idx: &i64,
    ) -> Step<'a, T> {
        pointer
            .as_array()
            .map(|array| {
                if (idx.abs() as usize) < array.len() {
                    let i = if *idx < 0 {
                        array.len() - idx.abs() as usize
                    } else {
                        *idx as usize
                    };
                    Step::new_ref(Data::new_idx(&array[i], path, i))
                } else {
                    Step::Nothing
                }
            })
            .unwrap_or_default()
    }
}

impl Query for Segment {
    fn process<'a, T: Queryable>(&self, step: Step<'a, T>) -> Step<'a, T> {
        match self {
            Segment::Descendant => {
                unimplemented!()
            }
            Segment::Selector(selector) => match selector {
                Selector::Name(key) => step.flat_map(|d| self.process_key(d, key)),
                Selector::Index(idx) => step.flat_map(|d| self.process_index(d, idx)),
                Selector::Wildcard => {
                    unimplemented!()
                }
                Selector::Slice(_, _, _) => {
                    unimplemented!()
                }
                Selector::Filter(_) => {
                    unimplemented!()
                }
            },
            Segment::Selectors(_) => {
                unimplemented!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_process_key() {
        let value = json!({"key": "value"});
        let segment = Segment::Selector(Selector::Name("key".to_string()));
        let step = segment.process(Step::new_ref(Data::new(&value, "$".to_string())));

        assert_eq!(
            step.ok(),
            Some(Data::new(&json!("value"), "$.['key']".to_string()))
        );
    }

    #[test]
    fn test_process_key_failed() {
        let value = json!({"key": "value"});
        let segment = Segment::Selector(Selector::Name("key2".to_string()));
        let step = segment.process(Step::new_ref(Data::new(&value, "$".to_string())));

        assert_eq!(step, Step::Nothing);
    }

    #[test]
    fn test_process_index() {
        let value = json!([1, 2, 3]);
        let segment = Segment::Selector(Selector::Index(1));
        let step = segment.process(Step::new_ref(Data::new(&value, "$".to_string())));

        assert_eq!(step.ok(), Some(Data::new(&json!(2), "$[1]".to_string())));
    }

    #[test]
    fn test_process_index_failed() {
        let value = json!([1, 2, 3]);
        let segment = Segment::Selector(Selector::Index(3));
        let step = segment.process(Step::new_ref(Data::new(&value, "$".to_string())));

        assert_eq!(step, Step::Nothing);
    }
}
