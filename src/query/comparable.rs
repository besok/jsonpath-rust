use crate::parser::model::{Comparable, Literal, SingularQuery, SingularQuerySegment};
use crate::query::queryable::Queryable;
use crate::query::selector::{process_index, process_key};
use crate::query::state::{Data, State};
use crate::query::Query;

impl Query for Comparable {
    fn process<'a, T: Queryable>(&self, step: State<'a, T>) -> State<'a, T> {
        match self {
            Comparable::Literal(lit) => lit.process(step),
            Comparable::Function(tf) => tf.process(step),
            Comparable::SingularQuery(query) => query.process(step),
        }
    }
}

impl Query for Literal {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        let val = match self {
            Literal::Int(v) => (*v).into(),
            Literal::Float(v) => (*v).into(),
            Literal::String(v) => v.as_str().into(),
            Literal::Bool(v) => (*v).into(),
            Literal::Null => T::null(),
        };

        State::data(state.root, Data::Value(val))
    }
}

impl Query for SingularQuery {
    fn process<'a, T: Queryable>(&self, step: State<'a, T>) -> State<'a, T> {
        match self {
            SingularQuery::Current(segments) => segments.process(step),
            SingularQuery::Root(segments) => segments.process(step.shift_to_root()),
        }
    }
}

impl Query for SingularQuerySegment {
    fn process<'a, T: Queryable>(&self, step: State<'a, T>) -> State<'a, T> {
        match self {
            SingularQuerySegment::Index(idx) => step.flat_map(|d| process_index(d, idx)),
            SingularQuerySegment::Name(key) => step.flat_map(|d| process_key(d, key)),
        }
    }
}

impl Query for Vec<SingularQuerySegment> {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        self.iter()
            .fold(state, |next, segment| segment.process(next))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::model::{Comparable, Literal, SingularQuery, SingularQuerySegment};
    use crate::query::state::{Data, Pointer, State};
    use crate::query::Query;
    use serde_json::json;

    #[test]
    fn singular_query() {
        let value = json!({
          "result": [
            {
              "message": "Hello, Emmy! Your order number is: #100",
              "phoneNumber": "255-301-9429",
              "phoneVariation": "+90 398 588 10 73",
              "status": "active",
              "name": {
                "first": "Blaise",
                "middle": "Kyle",
                "last": "Fadel"
              }
            }
          ]
        });

        let query = SingularQuery::Current(vec![
            SingularQuerySegment::Name("result".to_string()),
            SingularQuerySegment::Index(0),
            SingularQuerySegment::Name("name".to_string()),
            SingularQuerySegment::Name("first".to_string()),
        ]);

        let state = State::root(&value);

        let result = query.process(state);
        assert_eq!(
            result.ok_ref(),
            Some(vec![Pointer::new(
                &json!("Blaise"),
                "$['result'][0]['name']['first']".to_string()
            )])
        );
    }

    #[test]
    fn singular_query_root() {
        let value = json!({
          "result": [
            {
              "message": "Hello, Emmy! Your order number is: #100",
              "phoneNumber": "255-301-9429",
              "phoneVariation": "+90 398 588 10 73",
              "status": "active",
              "name": {
                "first": "Blaise",
                "middle": "Kyle",
                "last": "Fadel"
              }
            }
          ]
        });

        let query = SingularQuery::Root(vec![
            SingularQuerySegment::Name("result".to_string()),
            SingularQuerySegment::Index(0),
            SingularQuerySegment::Name("name".to_string()),
            SingularQuerySegment::Name("first".to_string()),
        ]);

        let state = State::data(
            &value,
            Data::new_ref(Pointer::new(&value, "$.name".to_string())),
        );

        let result = query.process(state);
        assert_eq!(
            result.ok_ref(),
            Some(vec![Pointer::new(
                &json!("Blaise"),
                "$['result'][0]['name']['first']".to_string()
            )])
        );
    }

    #[test]
    fn literal() {
        let value = json!({
          "result": [
            {
              "message": "Hello, Emmy! Your order number is: #100",
              "phoneNumber": "255-301-9429",
              "phoneVariation": "+90 398 588 10 73",
              "status": "active",
              "name": {
                "first": "Blaise",
                "middle": "Kyle",
                "last": "Fadel"
              }
            }
          ]
        });

        let query = Comparable::Literal(Literal::String("Hello".to_string()));

        let state = State::root(&value);

        let result = query.process(state);
        assert_eq!(result.ok_val(), Some(json!("Hello")));
    }
}
