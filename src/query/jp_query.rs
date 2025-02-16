use crate::parser::model2::JpQuery;
use crate::query::queryable::Queryable;
use crate::query::{Query, Step};

impl Query for JpQuery {
    fn process<'a, T: Queryable>(&self, step: Step<'a, T>) -> Step<'a, T> {
        self.segments
            .iter()
            .fold(step, |next, segment| segment.process(next))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::parser::model2::{JpQuery, Segment, Selector};
    use crate::query::{Data, Query, Step};

    #[test]
    fn test_process() {
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

        let query = JpQuery::new(vec![
            Segment::Selector(Selector::Name("result".to_string())),
            Segment::Selector(Selector::Index(0)),
            Segment::Selector(Selector::Name("name".to_string())),
            Segment::Selector(Selector::Name("first".to_string())),
        ]);

        let result = query.process(Step::new_ref(Data::new(&value, "$".to_string())));
        assert_eq!(
            result.ok(),
            Some(vec![Data::new(&json!("Blaise"), "$.['result'][0].['name'].['first']".to_string())])
        );
    }
}
