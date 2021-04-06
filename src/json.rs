use serde_json::Value;

pub fn inside(left: Vec<&Value>, right: Vec<&Value>) -> bool {
    if left.is_empty() {
        return false;
    }

    match right.get(0) {
        Some(Value::Array(elems)) => {
            for el in left.iter() {
                if elems.contains(el) {
                    return true;
                }
            }
            false
        }
        Some(Value::Object(elems)) => {
            for el in left.iter() {
                for r in elems.values() {
                    if el.eq(&r) {
                        return true;
                    }
                }
            }
            false
        }
        _ => false
    }
}

pub fn less(left: Vec<&Value>, right: Vec<&Value>) -> bool {
    if left.len() == 1 && right.len() == 1 {
        match (left.get(0), right.get(0)) {
            (Some(Value::Number(l)), Some(Value::Number(r))) =>
                l.as_f64().and_then(|v1| r.as_f64().map(|v2| v1 < v2)).unwrap_or(false),
            _ => false
        }
    } else {
        false
    }
}

pub fn eq(left: Vec<&Value>, right: Vec<&Value>) -> bool {
    if left.len() != right.len() {
        false
    } else {
        left.iter()
            .zip(right)
            .map(|(a, b)| a.eq(&b))
            .fold(true, |a, n| a && n)
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::json::{eq, less};

    #[test]
    fn value_eq_test() {
        let left = json!({"value":42});
        let right = json!({"value":42});
        let right_uneq = json!([42]);

        assert!(&left.eq(&right));
        assert!(!&left.eq(&right_uneq));
    }

    #[test]
    fn vec_value_test() {
        let left = json!({"value":42});
        let left1 = json!(42);
        let left2 = json!([1,2,3]);
        let left3 = json!({"value2":[42],"value":[42]});

        let right = json!({"value":42});
        let right1 = json!(42);
        let right2 = json!([1,2,3]);
        let right3 = json!({"value":[42],"value2":[42]});

        assert!(eq(vec![&left], vec![&right]));

        assert!(!eq(vec![], vec![&right]));
        assert!(!eq(vec![&right], vec![]));

        assert!(eq(vec![&left, &left1, &left2, &left3], vec![&right, &right1, &right2, &right3]));

        assert!(!eq(vec![&left1, &left, &left2, &left3], vec![&right, &right1, &right2, &right3]));
    }

    #[test]
    fn left_value_test() {
        let left = json!(10);
        let right = json!(11);

        assert!(less(vec![&left], vec![&right]));
        assert!(!less(vec![&right], vec![&left]));

        let left = json!(-10);
        let right = json!(-11);

        assert!(!less(vec![&left], vec![&right]));
        assert!(less(vec![&right], vec![&left]));

        let left = json!(-10.0);
        let right = json!(-11.0);

        assert!(!less(vec![&left], vec![&right]));
        assert!(less(vec![&right], vec![&left]));

        assert!(!less(vec![], vec![&right]));
        assert!(!less(vec![&right, &right], vec![&left]));
    }
}