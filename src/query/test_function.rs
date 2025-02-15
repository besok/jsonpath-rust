use crate::parser::model2::TestFunction;
use crate::query::{Step, Query};
use crate::query::queryable::Queryable;

impl TestFunction {
    pub fn apply<'a,T:Queryable>(&self, progress: Step<'a, T>) -> Step<'a,T>{

        match progress {
            // Progress::Data(data) => {
            //      match self {
            //          TestFunction::Custom(name, arg) => Progress::Nothing,
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