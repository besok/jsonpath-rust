use crate::parser::model2::Selector;
use crate::query::{Step, Query};
use crate::query::queryable::Queryable;

impl Query for Selector {
    fn process<'a, T: Queryable>(&self, progress: Step<'a, T>) -> Step<'a, T> {
        todo!()
    }
}