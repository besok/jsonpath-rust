use serde_json::{Result, Value};
use std::rc::Rc;


trait Step {
    fn next(&self, current_el: &Value) -> &Value;
}


trait Extractor {
    fn extract(&self, data: &Value, steps: Vec<dyn Step>) -> &Value {
        steps.iter().fold(data, |next_data, step| step.next(next_data))
    }
}