use crate::parser::model2::Selector;
use crate::query::queryable::Queryable;
use crate::query::Query;
use std::cmp::{max, min};
use crate::query::state::{Data, Pointer, State};

impl Query for Selector {
    fn process<'a, T: Queryable>(&self, step: State<'a, T>) -> State<'a, T> {
        match self {
            Selector::Name(key) => step.flat_map(|d| process_key(d, key)),
            Selector::Index(idx) => step.flat_map(|d| process_index(d, idx)),
            Selector::Wildcard => step.flat_map(process_wildcard),
            Selector::Slice(start, end, sl_step) => {
                step.flat_map(|d| process_slice(d, start, end, sl_step))
            }
            Selector::Filter(_) => {
                unimplemented!()
            }
        }
    }
}

fn process_wildcard<T: Queryable>(Pointer { inner: pointer, path }: Pointer<T>) -> Data<T> {
    if let Some(array) = pointer.as_array() {
        Data::new_refs(
            array
                .iter()
                .enumerate()
                .map(|(i, elem)| Pointer::idx(elem, path.clone(), i))
                .collect(),
        )
    } else if let Some(object) = pointer.as_object() {
        Data::new_refs(
            object
                .into_iter()
                .map(|(key, value)| Pointer::key(value, path.clone(), key))
                .collect(),
        )
    } else {
        Data::Nothing
    }
}

fn process_slice<'a, T: Queryable>(
    Pointer { inner, path }: Pointer<'a, T>,
    start: &Option<i64>,
    end: &Option<i64>,
    step: &Option<i64>,
) -> Data<'a, T> {
    let extract_elems = |elements: &'a Vec<T>| -> Vec<(&'a T, usize)> {
        let len = elements.len() as i64;
        let norm = |i: i64| {
            if i >= 0 {
                i
            } else {
                len + i
            }
        };

        match step.unwrap_or(1) {
            e if e > 0 => {
                let n_start = norm(start.unwrap_or(0));
                let n_end = norm(end.unwrap_or(len));
                let lower = min(max(n_start, 0), len);
                let upper = min(max(n_end, 0), len);

                let mut idx = lower;
                let mut res = vec![];
                while idx < upper {
                    let i = idx as usize;
                    if let Some(elem) = elements.get(i) {
                        res.push((elem, i));
                    }
                    idx += e;
                }
                res
            }
            e if e < 0 => {
                let n_start = norm(start.unwrap_or(len - 1));
                let n_end = norm(end.unwrap_or(-len - 1));
                let lower = min(max(n_end, -1), len - 1);
                let upper = min(max(n_start, -1), len - 1);
                let mut idx = upper;
                let mut res = vec![];
                while lower < idx {
                    let i = idx as usize;
                    if let Some(elem) = elements.get(i) {
                        res.push((elem, i));
                    }
                    idx += e;
                }
                res
            }
            _ => vec![],
        }
    };

    let elems_to_step = |v: Vec<(&'a T, usize)>| {
        Data::new_refs(
            v.into_iter()
                .map(|(elem, i)| Pointer::idx(elem, path.clone(), i))
                .collect(),
        )
    };

    inner
        .as_array()
        .map(extract_elems)
        .map(elems_to_step)
        .unwrap_or_default()
}

pub fn process_key<'a, T: Queryable>(Pointer { inner, path }: Pointer<'a, T>, key: &str) -> Data<'a, T> {
    inner
        .get(key)
        .map(|v| Data::new_ref(Pointer::key(v, path, key)))
        .unwrap_or_default()
}

pub fn process_index<'a, T: Queryable>(Pointer { inner, path }: Pointer<'a, T>, idx: &i64) -> Data<'a, T> {
    inner
        .as_array()
        .map(|array| {
            if (idx.abs() as usize) < array.len() {
                let i = if *idx < 0 {
                    array.len() - idx.abs() as usize
                } else {
                    *idx as usize
                };
                Data::new_ref(Pointer::idx(&array[i], path, i))
            } else {
                Data::Nothing
            }
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::model2::Segment;
    use serde_json::json;
    use std::vec;

    #[test]
    fn test_process_key() {
        let value = json!({"key": "value"});
        let segment = Segment::Selector(Selector::Name("key".to_string()));

        let step = segment.process(State::root(&value));

        assert_eq!(
            step.ok(),
            Some(vec![Pointer::new(&json!("value"), "$.['key']".to_string())])
        );
    }

    #[test]
    fn test_process_key_failed() {
        let value = json!({"key": "value"});
        let segment = Segment::Selector(Selector::Name("key2".to_string()));
        let step = segment.process(State::root(&value));

        assert_eq!(step, State::new(&value));
    }

    #[test]
    fn test_process_index() {
        let value = json!([1, 2, 3]);
        let segment = Segment::Selector(Selector::Index(1));
        let step = segment.process(State::root(&value));

        assert_eq!(
            step.ok(),
            Some(vec![Pointer::new(&json!(2), "$[1]".to_string())])
        );
    }

    #[test]
    fn test_process_index_failed() {
        let value = json!([1, 2, 3]);
        let segment = Segment::Selector(Selector::Index(3));
        let step = segment.process(State::root(&value));

        assert_eq!(step, State::new(&value));
    }

    #[test]
    fn test_process_slice1() {
        let value = json!([1, 2, 3, 4, 5]);
        let segment = Segment::Selector(Selector::Slice(Some(1), Some(4), Some(1)));
        let step = segment.process(State::root(&value));

        assert_eq!(
            step.ok(),
            Some(vec![
                Pointer::new(&json!(2), "$[1]".to_string()),
                Pointer::new(&json!(3), "$[2]".to_string()),
                Pointer::new(&json!(4), "$[3]".to_string())
            ])
        );
    }

    #[test]
    fn test_process_slice2() {
        let value = json!([1, 2, 3, 4, 5]);
        let segment = Segment::Selector(Selector::Slice(Some(2), Some(0), Some(-1)));
        let step = segment.process(State::root(&value));

        assert_eq!(
            step.ok(),
            Some(vec![
                Pointer::new(&json!(3), "$[2]".to_string()),
                Pointer::new(&json!(2), "$[1]".to_string()),
            ])
        );
    }

    #[test]
    fn test_process_slice3() {
        let value = json!([1, 2, 3, 4, 5]);
        let segment = Segment::Selector(Selector::Slice(Some(0), Some(5), Some(2)));
        let step = segment.process(State::root(&value));

        assert_eq!(
            step.ok(),
            Some(vec![
                Pointer::new(&json!(1), "$[0]".to_string()),
                Pointer::new(&json!(3), "$[2]".to_string()),
                Pointer::new(&json!(5), "$[4]".to_string())
            ])
        );
    }

    #[test]
    fn test_process_slice_failed() {
        let value = json!([1, 2, 3, 4, 5]);
        let segment = Segment::Selector(Selector::Slice(Some(0), Some(5), Some(0)));
        let step = segment.process(State::root(&value));

        assert_eq!(step.ok(), Some(vec![]));
    }

    #[test]
    fn test_process_wildcard() {
        let value = json!({"key": "value", "key2": "value2"});
        let segment = Segment::Selector(Selector::Wildcard);
        let step = segment.process(State::root(&value));

        assert_eq!(
            step.ok(),
            Some(vec![
                Pointer::new(&json!("value"), "$.['key']".to_string()),
                Pointer::new(&json!("value2"), "$.['key2']".to_string())
            ])
        );
    }

    #[test]
    fn test_process_wildcard_array() {
        let value = json!([1, 2, 3]);
        let segment = Segment::Selector(Selector::Wildcard);
        let step = segment.process(State::root(&value));

        assert_eq!(
            step.ok(),
            Some(vec![
                Pointer::new(&json!(1), "$[0]".to_string()),
                Pointer::new(&json!(2), "$[1]".to_string()),
                Pointer::new(&json!(3), "$[2]".to_string())
            ])
        );
    }

    #[test]
    fn test_process_wildcard_failed() {
        let value = json!(1);
        let segment = Segment::Selector(Selector::Wildcard);
        let step = segment.process(State::root(&value));

        assert_eq!(step, State::new(&value));
    }
}
