use crate::path::json_path_instance;
use crate::path::JsonLike;
use crate::JsonPathValue;
use crate::JsonPtr;
use crate::{JsonPath, JsonPathStr};

impl<T> JsonPath<T>
where
    T: JsonLike,
{
    /// finds a slice of data in the set json.
    /// The result is a vector of references to the incoming structure.
    ///
    /// In case, if there is no match [`Self::find_slice`] will return vec!<[`JsonPathValue::NoValue`]>.
    ///
    /// ## Example
    /// ```rust
    /// use jsonpath_rust::{JsonPath, JsonPathValue};
    /// use serde_json::json;
    /// # use std::str::FromStr;
    ///
    /// let data = json!({"first":{"second":[{"active":1},{"passive":1}]}});
    /// let path = JsonPath::try_from("$.first.second[?(@.active)]").unwrap();
    /// let slice_of_data = path.find_slice(&data);
    ///
    /// let expected_value = json!({"active":1});
    /// let expected_path = "$.['first'].['second'][0]".to_string();
    ///
    /// assert_eq!(
    ///     slice_of_data,
    ///     vec![JsonPathValue::Slice(&expected_value, expected_path)]
    /// );
    /// ```
    pub fn find_slice<'a>(&'a self, json: &'a T) -> Vec<JsonPathValue<'a, T>> {
        use crate::path::Path;
        json_path_instance(self, json)
            .find(JsonPathValue::from_root(json))
            .into_iter()
            .filter(|v| v.has_value())
            .collect()
    }

    /// like [`Self::find_slice`] but returns a vector of [`JsonPtr`], which has no [`JsonPathValue::NoValue`].
    /// if there is no match, it will return an empty vector
    pub fn find_slice_ptr<'a>(&'a self, json: &'a T) -> Vec<JsonPtr<'a, T>> {
        use crate::path::Path;
        json_path_instance(self, json)
            .find(JsonPathValue::from_root(json))
            .into_iter()
            .filter(|v| v.has_value())
            .map(|v| match v {
                JsonPathValue::Slice(v, _) => JsonPtr::Slice(v),
                JsonPathValue::NewValue(v) => JsonPtr::NewValue(v),
                JsonPathValue::NoValue => unreachable!("has_value was already checked"),
            })
            .collect()
    }

    /// finds a slice of data and wrap it with Value::Array by cloning the data.
    /// Returns either an array of elements or Json::Null if the match is incorrect.
    ///
    /// In case, if there is no match `find` will return `json!(null)`.
    ///
    /// ## Example
    /// ```rust
    /// use jsonpath_rust::{JsonPath, JsonPathValue};
    /// use serde_json::{Value, json};
    /// # use std::str::FromStr;
    ///
    /// let data = json!({"first":{"second":[{"active":1},{"passive":1}]}});
    /// let path = JsonPath::try_from("$.first.second[?(@.active)]").unwrap();
    /// let cloned_data = path.find(&data);
    ///
    /// assert_eq!(cloned_data, Value::Array(vec![json!({"active":1})]));
    /// ```
    pub fn find(&self, json: &T) -> T {
        let slice = self.find_slice(json);
        if !slice.is_empty() {
            if JsonPathValue::only_no_value(&slice) {
                T::null()
            } else {
                T::array(
                    slice
                        .into_iter()
                        .filter(|v| v.has_value())
                        .map(|v| v.to_data())
                        .collect(),
                )
            }
        } else {
            T::array(vec![])
        }
    }

    /// finds a path describing the value, instead of the value itself.
    /// If the values has been obtained by moving the data out of the initial json the path is absent.
    ///
    /// ** If the value has been modified during the search, there is no way to find a path of a new value.
    /// It can happen if we try to find a length() of array, for in stance.**
    ///
    /// ## Example
    /// ```rust
    /// use jsonpath_rust::{JsonPathStr, JsonPath, JsonPathValue};
    /// use serde_json::{Value, json};
    /// # use std::str::FromStr;
    ///
    /// let data = json!({"first":{"second":[{"active":1},{"passive":1}]}});
    /// let path = JsonPath::try_from("$.first.second[?(@.active)]").unwrap();
    /// let slice_of_data: Vec<JsonPathStr> = path.find_as_path(&data);
    ///
    /// let expected_path = "$.['first'].['second'][0]".to_string();
    /// assert_eq!(slice_of_data, vec![expected_path]);
    /// ```
    pub fn find_as_path(&self, json: &T) -> Vec<JsonPathStr> {
        self.find_slice(json)
            .into_iter()
            .flat_map(|v| v.to_path())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::path::JsonLike;
    use crate::JsonPathQuery;
    use crate::JsonPathValue::{NoValue, Slice};
    use crate::{jp_v, JsonPath, JsonPathParserError, JsonPathValue};
    use serde_json::{json, Value};
    use std::ops::Deref;

    fn test(json: &str, path: &str, expected: Vec<JsonPathValue<Value>>) {
        let json: Value = match serde_json::from_str(json) {
            Ok(json) => json,
            Err(e) => panic!("error while parsing json: {}", e),
        };
        let path = match JsonPath::try_from(path) {
            Ok(path) => path,
            Err(e) => panic!("error while parsing jsonpath: {}", e),
        };

        assert_eq!(path.find_slice(&json), expected)
    }

    fn template_json<'a>() -> &'a str {
        r#" {"store": { "book": [
             {
                 "category": "reference",
                 "author": "Nigel Rees",
                 "title": "Sayings of the Century",
                 "price": 8.95
             },
             {
                 "category": "fiction",
                 "author": "Evelyn Waugh",
                 "title": "Sword of Honour",
                 "price": 12.99
             },
             {
                 "category": "fiction",
                 "author": "Herman Melville",
                 "title": "Moby Dick",
                 "isbn": "0-553-21311-3",
                 "price": 8.99
             },
             {
                 "category": "fiction",
                 "author": "J. R. R. Tolkien",
                 "title": "The Lord of the Rings",
                 "isbn": "0-395-19395-8",
                 "price": 22.99
             }
         ],
         "bicycle": {
             "color": "red",
             "price": 19.95
         }
     },
     "array":[0,1,2,3,4,5,6,7,8,9],
     "orders":[
         {
             "ref":[1,2,3],
             "id":1,
             "filled": true
         },
         {
             "ref":[4,5,6],
             "id":2,
             "filled": false
         },
         {
             "ref":[7,8,9],
             "id":3,
             "filled": null
         }
      ],
     "expensive": 10 }"#
    }

    #[test]
    fn simple_test() {
        let j1 = json!(2);
        test("[1,2,3]", "$[1]", jp_v![&j1;"$[1]",]);
    }

    #[test]
    fn root_test() {
        let js = serde_json::from_str(template_json()).unwrap();
        test(template_json(), "$", jp_v![&js;"$",]);
    }

    #[test]
    fn descent_test() {
        let v1 = json!("reference");
        let v2 = json!("fiction");
        test(
            template_json(),
            "$..category",
            jp_v![
                 &v1;"$.['store'].['book'][0].['category']",
                 &v2;"$.['store'].['book'][1].['category']",
                 &v2;"$.['store'].['book'][2].['category']",
                 &v2;"$.['store'].['book'][3].['category']",],
        );
        let js1 = json!(19.95);
        let js2 = json!(8.95);
        let js3 = json!(12.99);
        let js4 = json!(8.99);
        let js5 = json!(22.99);
        test(
            template_json(),
            "$.store..price",
            jp_v![
                &js1;"$.['store'].['bicycle'].['price']",
                &js2;"$.['store'].['book'][0].['price']",
                &js3;"$.['store'].['book'][1].['price']",
                &js4;"$.['store'].['book'][2].['price']",
                &js5;"$.['store'].['book'][3].['price']",
            ],
        );
        let js1 = json!("Nigel Rees");
        let js2 = json!("Evelyn Waugh");
        let js3 = json!("Herman Melville");
        let js4 = json!("J. R. R. Tolkien");
        test(
            template_json(),
            "$..author",
            jp_v![
            &js1;"$.['store'].['book'][0].['author']",
            &js2;"$.['store'].['book'][1].['author']",
            &js3;"$.['store'].['book'][2].['author']",
            &js4;"$.['store'].['book'][3].['author']",],
        );
    }

    #[test]
    fn wildcard_test() {
        let js1 = json!("reference");
        let js2 = json!("fiction");
        test(
            template_json(),
            "$..book.[*].category",
            jp_v![
                &js1;"$.['store'].['book'][0].['category']",
                &js2;"$.['store'].['book'][1].['category']",
                &js2;"$.['store'].['book'][2].['category']",
                &js2;"$.['store'].['book'][3].['category']",],
        );
        let js1 = json!("Nigel Rees");
        let js2 = json!("Evelyn Waugh");
        let js3 = json!("Herman Melville");
        let js4 = json!("J. R. R. Tolkien");
        test(
            template_json(),
            "$.store.book[*].author",
            jp_v![
                &js1;"$.['store'].['book'][0].['author']",
                &js2;"$.['store'].['book'][1].['author']",
                &js3;"$.['store'].['book'][2].['author']",
                &js4;"$.['store'].['book'][3].['author']",],
        );
    }

    #[test]
    fn descendent_wildcard_test() {
        let js1 = json!("Moby Dick");
        let js2 = json!("The Lord of the Rings");
        test(
            template_json(),
            "$..*.[?(@.isbn)].title",
            jp_v![
                &js1;"$.['store'].['book'][2].['title']",
                &js2;"$.['store'].['book'][3].['title']",
                &js1;"$.['store'].['book'][2].['title']",
                &js2;"$.['store'].['book'][3].['title']"],
        );
    }

    #[test]
    fn field_test() {
        let value = json!({"active":1});
        test(
            r#"{"field":{"field":[{"active":1},{"passive":1}]}}"#,
            "$.field.field[?(@.active)]",
            jp_v![&value;"$.['field'].['field'][0]",],
        );
    }

    #[test]
    fn index_index_test() {
        let value = json!("0-553-21311-3");
        test(
            template_json(),
            "$..book[2].isbn",
            jp_v![&value;"$.['store'].['book'][2].['isbn']",],
        );
    }

    #[test]
    fn index_unit_index_test() {
        let value = json!("0-553-21311-3");
        test(
            template_json(),
            "$..book[2,4].isbn",
            jp_v![&value;"$.['store'].['book'][2].['isbn']",],
        );
        let value1 = json!("0-395-19395-8");
        test(
            template_json(),
            "$..book[2,3].isbn",
            jp_v![&value;"$.['store'].['book'][2].['isbn']", &value1;"$.['store'].['book'][3].['isbn']",],
        );
    }

    #[test]
    fn index_unit_keys_test() {
        let js1 = json!("Moby Dick");
        let js2 = json!(8.99);
        let js3 = json!("The Lord of the Rings");
        let js4 = json!(22.99);
        test(
            template_json(),
            "$..book[2,3]['title','price']",
            jp_v![
                &js1;"$.['store'].['book'][2].['title']",
                &js2;"$.['store'].['book'][2].['price']",
                &js3;"$.['store'].['book'][3].['title']",
                &js4;"$.['store'].['book'][3].['price']",],
        );
    }

    #[test]
    fn index_slice_test() {
        let i0 = "$.['array'][0]";
        let i1 = "$.['array'][1]";
        let i2 = "$.['array'][2]";
        let i3 = "$.['array'][3]";
        let i4 = "$.['array'][4]";
        let i5 = "$.['array'][5]";
        let i6 = "$.['array'][6]";
        let i7 = "$.['array'][7]";
        let i8 = "$.['array'][8]";
        let i9 = "$.['array'][9]";

        let j0 = json!(0);
        let j1 = json!(1);
        let j2 = json!(2);
        let j3 = json!(3);
        let j4 = json!(4);
        let j5 = json!(5);
        let j6 = json!(6);
        let j7 = json!(7);
        let j8 = json!(8);
        let j9 = json!(9);
        test(
            template_json(),
            "$.array[:]",
            jp_v![
                &j0;&i0,
                &j1;&i1,
                &j2;&i2,
                &j3;&i3,
                &j4;&i4,
                &j5;&i5,
                &j6;&i6,
                &j7;&i7,
                &j8;&i8,
                &j9;&i9,],
        );
        test(template_json(), "$.array[1:4:2]", jp_v![&j1;&i1, &j3;&i3,]);
        test(
            template_json(),
            "$.array[::3]",
            jp_v![&j0;&i0, &j3;&i3, &j6;&i6, &j9;&i9,],
        );
        test(template_json(), "$.array[-1:]", jp_v![&j9;&i9,]);
        test(template_json(), "$.array[-2:-1]", jp_v![&j8;&i8,]);
    }

    #[test]
    fn index_filter_test() {
        let moby = json!("Moby Dick");
        let rings = json!("The Lord of the Rings");
        test(
            template_json(),
            "$..book[?(@.isbn)].title",
            jp_v![
                &moby;"$.['store'].['book'][2].['title']",
                &rings;"$.['store'].['book'][3].['title']",],
        );
        let sword = json!("Sword of Honour");
        test(
            template_json(),
            "$..book[?(@.price != 8.95)].title",
            jp_v![
                &sword;"$.['store'].['book'][1].['title']",
                &moby;"$.['store'].['book'][2].['title']",
                &rings;"$.['store'].['book'][3].['title']",],
        );
        let sayings = json!("Sayings of the Century");
        test(
            template_json(),
            "$..book[?(@.price == 8.95)].title",
            jp_v![&sayings;"$.['store'].['book'][0].['title']",],
        );
        let js895 = json!(8.95);
        test(
            template_json(),
            "$..book[?(@.author ~= '.*Rees')].price",
            jp_v![&js895;"$.['store'].['book'][0].['price']",],
        );
        let js12 = json!(12.99);
        let js899 = json!(8.99);
        let js2299 = json!(22.99);
        test(
            template_json(),
            "$..book[?(@.price >= 8.99)].price",
            jp_v![
                &js12;"$.['store'].['book'][1].['price']",
                &js899;"$.['store'].['book'][2].['price']",
                &js2299;"$.['store'].['book'][3].['price']",
            ],
        );
        test(
            template_json(),
            "$..book[?(@.price > 8.99)].price",
            jp_v![
                &js12;"$.['store'].['book'][1].['price']",
                &js2299;"$.['store'].['book'][3].['price']",],
        );
        test(
            template_json(),
            "$..book[?(@.price < 8.99)].price",
            jp_v![&js895;"$.['store'].['book'][0].['price']",],
        );
        test(
            template_json(),
            "$..book[?(@.price <= 8.99)].price",
            jp_v![
                &js895;"$.['store'].['book'][0].['price']",
                &js899;"$.['store'].['book'][2].['price']",
            ],
        );
        test(
            template_json(),
            "$..book[?(@.price <= $.expensive)].price",
            jp_v![
                &js895;"$.['store'].['book'][0].['price']",
                &js899;"$.['store'].['book'][2].['price']",
            ],
        );
        test(
            template_json(),
            "$..book[?(@.price >= $.expensive)].price",
            jp_v![
                &js12;"$.['store'].['book'][1].['price']",
                &js2299;"$.['store'].['book'][3].['price']",
            ],
        );
        test(
            template_json(),
            "$..book[?(@.title in ['Moby Dick','Shmoby Dick','Big Dick','Dicks'])].price",
            jp_v![&js899;"$.['store'].['book'][2].['price']",],
        );
        test(
            template_json(),
            "$..book[?(@.title nin ['Moby Dick','Shmoby Dick','Big Dick','Dicks'])].title",
            jp_v![
                &sayings;"$.['store'].['book'][0].['title']",
                &sword;"$.['store'].['book'][1].['title']",
                &rings;"$.['store'].['book'][3].['title']",],
        );
        test(
            template_json(),
            "$..book[?(@.author size 10)].title",
            jp_v![&sayings;"$.['store'].['book'][0].['title']",],
        );
        let filled_true = json!(1);
        test(
            template_json(),
            "$.orders[?(@.filled == true)].id",
            jp_v![&filled_true;"$.['orders'][0].['id']",],
        );
        let filled_null = json!(3);
        test(
            template_json(),
            "$.orders[?(@.filled == null)].id",
            jp_v![&filled_null;"$.['orders'][2].['id']",],
        );
    }

    #[test]
    fn index_filter_sets_test() {
        let j1 = json!(1);
        test(
            template_json(),
            "$.orders[?(@.ref subsetOf [1,2,3,4])].id",
            jp_v![&j1;"$.['orders'][0].['id']",],
        );
        let j2 = json!(2);
        test(
            template_json(),
            "$.orders[?(@.ref anyOf [1,4])].id",
            jp_v![&j1;"$.['orders'][0].['id']", &j2;"$.['orders'][1].['id']",],
        );
        let j3 = json!(3);
        test(
            template_json(),
            "$.orders[?(@.ref noneOf [3,6])].id",
            jp_v![&j3;"$.['orders'][2].['id']",],
        );
    }

    #[test]
    fn query_test() {
        let json: Box<Value> = serde_json::from_str(template_json()).expect("to get json");
        let v = json
            .path("$..book[?(@.author size 10)].title")
            .expect("the path is correct");
        assert_eq!(v, json!(["Sayings of the Century"]));

        let json: Value = serde_json::from_str(template_json()).expect("to get json");
        let path = &json
            .path("$..book[?(@.author size 10)].title")
            .expect("the path is correct");

        assert_eq!(path, &json!(["Sayings of the Century"]));
    }

    #[test]
    fn find_slice_test() {
        let json: Box<Value> = serde_json::from_str(template_json()).expect("to get json");
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$..book[?(@.author size 10)].title").expect("the path is correct"),
        );
        let v = path.find_slice(&json);
        let js = json!("Sayings of the Century");
        assert_eq!(v, jp_v![&js;"$.['store'].['book'][0].['title']",]);
    }

    #[test]
    fn find_in_array_test() {
        let json: Box<Value> = Box::new(json!([{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.[?(@.verb == 'TEST')]").expect("the path is correct"));
        let v = path.find_slice(&json);
        let js = json!({"verb":"TEST"});
        assert_eq!(v, jp_v![&js;"$[0]",]);
    }

    #[test]
    fn length_test() {
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.[?(@.verb == 'TEST')].length()").expect("the path is correct"),
        );
        let v = path.find(&json);
        let js = json!([2]);
        assert_eq!(v, js);

        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.length()").expect("the path is correct"));
        assert_eq!(path.find(&json), json!([3]));

        // length of search following the wildcard returns correct result
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST","x":3}, {"verb": "RUN"}]));
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.[?(@.verb == 'TEST')].[*].length()")
                .expect("the path is correct"),
        );
        assert_eq!(path.find(&json), json!([3]));

        // length of object returns 0
        let json: Box<Value> = Box::new(json!({"verb": "TEST"}));
        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.length()").expect("the path is correct"));
        assert_eq!(path.find(&json), json!([]));

        // length of integer returns null
        let json: Box<Value> = Box::new(json!(1));
        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.length()").expect("the path is correct"));
        assert_eq!(path.find(&json), json!([]));

        // length of array returns correct result
        let json: Box<Value> = Box::new(json!([[1], [2], [3]]));
        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.length()").expect("the path is correct"));
        assert_eq!(path.find(&json), json!([3]));

        // path does not exist returns length null
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.not.exist.length()").expect("the path is correct"));
        assert_eq!(path.find(&json), json!([]));

        // seraching one value returns correct length
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.[?(@.verb == 'RUN')].length()").expect("the path is correct"),
        );

        let v = path.find(&json);
        let js = json!([1]);
        assert_eq!(v, js);

        // searching correct path following unexisting key returns length 0
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.[?(@.verb == 'RUN')].key123.length()")
                .expect("the path is correct"),
        );

        let v = path.find(&json);
        let js = json!([]);
        assert_eq!(v, js);

        // fetching first object returns length null
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.[0].length()").expect("the path is correct"));

        let v = path.find(&json);
        let js = json!([]);
        assert_eq!(v, js);

        // length on fetching the index after search gives length of the object (array)
        let json: Box<Value> = Box::new(json!([{"prop": [["a", "b", "c"], "d"]}]));
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.[?(@.prop)].prop.[0].length()").expect("the path is correct"),
        );

        let v = path.find(&json);
        let js = json!([3]);
        assert_eq!(v, js);

        // length on fetching the index after search gives length of the object (string)
        let json: Box<Value> = Box::new(json!([{"prop": [["a", "b", "c"], "d"]}]));
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.[?(@.prop)].prop.[1].length()").expect("the path is correct"),
        );

        let v = path.find(&json);
        let js = json!([]);
        assert_eq!(v, js);
    }

    #[test]
    fn no_value_index_from_not_arr_filter_test() {
        let json: Box<Value> = Box::new(json!({
            "field":"field",
        }));

        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.field[1]").expect("the path is correct"));
        let v = path.find_slice(&json);
        assert_eq!(v, vec![]);

        let json: Box<Value> = Box::new(json!({
            "field":[0],
        }));

        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.field[1]").expect("the path is correct"));
        let v = path.find_slice(&json);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn no_value_filter_from_not_arr_filter_test() {
        let json: Box<Value> = Box::new(json!({
            "field":"field",
        }));

        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.field[?(@ == 0)]").expect("the path is correct"));
        let v = path.find_slice(&json);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn no_value_index_filter_test() {
        let json: Box<Value> = Box::new(json!({
            "field":[{"f":1},{"f":0}],
        }));

        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.field[?(@.f_ == 0)]").expect("the path is correct"));
        let v = path.find_slice(&json);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn no_value_decent_test() {
        let json: Box<Value> = Box::new(json!({
            "field":[{"f":1},{"f":{"f_":1}}],
        }));

        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$..f_").expect("the path is correct"));
        let v = path.find_slice(&json);
        assert_eq!(
            v,
            vec![Slice(&json!(1), "$.['field'][1].['f'].['f_']".to_string())]
        );
    }

    #[test]
    fn no_value_chain_test() {
        let json: Box<Value> = Box::new(json!({
            "field":{"field":[1]},
        }));

        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.field_.field").expect("the path is correct"));
        let v = path.find_slice(&json);
        assert_eq!(v, vec![]);

        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.field_.field[?(@ == 1)]").expect("the path is correct"),
        );
        let v = path.find_slice(&json);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn no_value_filter_test() {
        // searching unexisting value returns length 0
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.[?(@.verb == \"RUN1\")]").expect("the path is correct"),
        );
        assert_eq!(path.find(&json), json!([]));
    }

    #[test]
    fn no_value_len_test() {
        let json: Box<Value> = Box::new(json!({
            "field":{"field":1},
        }));

        let path: Box<JsonPath<Value>> =
            Box::from(JsonPath::try_from("$.field.field.length()").expect("the path is correct"));
        let v = path.find_slice(&json);
        assert_eq!(v, vec![]);

        let json: Box<Value> = Box::new(json!({
            "field":[{"a":1},{"a":1}],
        }));
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.field[?(@.a == 0)].f.length()").expect("the path is correct"),
        );
        let v = path.find_slice(&json);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn no_clone_api_test() {
        fn test_coercion(value: &Value) -> Value {
            value.clone()
        }

        let json: Value = serde_json::from_str(template_json()).expect("to get json");
        let query =
            JsonPath::try_from("$..book[?(@.author size 10)].title").expect("the path is correct");

        let results = query.find_slice_ptr(&json);
        let v = results.first().expect("to get value");

        // V can be implicitly converted to &Value
        test_coercion(v);

        // To explicitly convert to &Value, use deref()
        assert_eq!(v.deref(), &json!("Sayings of the Century"));
    }

    #[test]
    fn logical_exp_test() {
        let json: Box<Value> = Box::new(json!({"first":{"second":[{"active":1},{"passive":1}]}}));

        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.first[?(@.does_not_exist && @.does_not_exist >= 1.0)]")
                .expect("the path is correct"),
        );
        let v = path.find_slice(&json);
        assert_eq!(v, vec![]);

        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.first[?(@.does_not_exist >= 1.0)]").expect("the path is correct"),
        );
        let v = path.find_slice(&json);
        assert_eq!(v, vec![]);
    }

    #[test]
    fn regex_filter_test() {
        let json: Box<Value> = Box::new(json!({
            "author":"abcd(Rees)",
        }));

        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.[?(@.author ~= '(?i)d\\(Rees\\)')]")
                .expect("the path is correct"),
        );
        assert_eq!(
            path.find_slice(&json.clone()),
            vec![Slice(&json!({"author":"abcd(Rees)"}), "$".to_string())]
        );
    }

    #[test]
    fn logical_not_exp_test() {
        let json: Box<Value> = Box::new(json!({"first":{"second":{"active":1}}}));
        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.first[?(!@.does_not_exist >= 1.0)]")
                .expect("the path is correct"),
        );
        let v = path.find_slice(&json);
        assert_eq!(
            v,
            vec![Slice(
                &json!({"second":{"active": 1}}),
                "$.['first']".to_string()
            )]
        );

        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.first[?(!(@.does_not_exist >= 1.0))]")
                .expect("the path is correct"),
        );
        let v = path.find_slice(&json);
        assert_eq!(
            v,
            vec![Slice(
                &json!({"second":{"active": 1}}),
                "$.['first']".to_string()
            )]
        );

        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.first[?(!(@.second.active == 1) || @.second.active == 1)]")
                .expect("the path is correct"),
        );
        let v = path.find_slice(&json);
        assert_eq!(
            v,
            vec![Slice(
                &json!({"second":{"active": 1}}),
                "$.['first']".to_string()
            )]
        );

        let path: Box<JsonPath<Value>> = Box::from(
            JsonPath::try_from("$.first[?(!@.second.active == 1 && !@.second.active == 1 || !@.second.active == 2)]")
                .expect("the path is correct"),
        );
        let v = path.find_slice(&json);
        assert_eq!(
            v,
            vec![Slice(
                &json!({"second":{"active": 1}}),
                "$.['first']".to_string()
            )]
        );
    }

    #[test]
    fn update_by_path_test() -> Result<(), JsonPathParserError> {
        let mut json = json!([
            {"verb": "RUN","distance":[1]},
            {"verb": "TEST"},
            {"verb": "DO NOT RUN"}
        ]);

        let path: Box<JsonPath> = Box::from(JsonPath::try_from("$.[?(@.verb == 'RUN')]")?);
        let elem = path
            .find_as_path(&json)
            .first()
            .cloned()
            .ok_or(JsonPathParserError::InvalidJsonPath("".to_string()))?;

        if let Some(v) = json
            .reference_mut(elem)?
            .and_then(|v| v.as_object_mut())
            .and_then(|v| v.get_mut("distance"))
            .and_then(|v| v.as_array_mut())
        {
            v.push(json!(2))
        }

        assert_eq!(
            json,
            json!([
                {"verb": "RUN","distance":[1,2]},
                {"verb": "TEST"},
                {"verb": "DO NOT RUN"}
            ])
        );

        Ok(())
    }
}
