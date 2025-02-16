use serde_json::json;
use crate::parser::model2::TestFunction;
use crate::query::{Step, Query};
use crate::query::queryable::Queryable;

impl TestFunction {
    /// Returns the length/size of the object.
    ///
    /// # Returns
    ///
    /// Returns a `Progress` enum containing either:
    /// - `Progress::Data` with a vector of references to self and the query path for strings/arrays/objects
    /// - `Progress::Nothing` for other types
    ///
    /// The returned length follows JSON path length() function semantics based on the type:
    /// - String type: Number of Unicode scalar values
    /// - Array type: Number of elements
    /// - Object type: Number of members
    /// - Other types: Nothing
    // fn length<'a, T:Queryable>(&self) -> Step<'a, T> {
    //     if let Some(str) = self.as_str() {
    //         Step::Value(json!(str.chars().count()))
    //     } else if let Some(arr) = self.as_array() {
    //         Step::Value(json!(arr.len()))
    //     } else if let Some(obj) = self.as_object() {
    //         Step::Value(json!(obj.len()))
    //     } else {
    //         Step::Nothing
    //     }
    // }

    pub fn apply<'a,T:Queryable>(&self, progress: Step<'a, T>) -> Step<'a,T>{

        match progress {
            // Step::Data(data) => {
            //      match self {
            //          TestFunction::Custom(name, arg) => Step::Nothing,
            //          TestFunction::Length(arg) => {}
            //          TestFunction::Value(_) => {}
            //          TestFunction::Count(_) => {}
            //          TestFunction::Search(_, _) => {}
            //          TestFunction::Match(_, _) => {}
            //      }
            // }
            _ => Step::Nothing
        }
    }
}

impl Query for TestFunction {
    fn process<'a, T: Queryable>(&self, progress: Step<'a, T>) -> Step<'a, T> {
        self.apply(progress)
    }
}