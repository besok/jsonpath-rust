use crate::parser::model::Test;
use crate::query::queryable::Queryable;
use crate::query::state::State;
use crate::query::Query;

impl Query for Test {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        match self {
            Test::RelQuery(segments) => segments.process(state),
            Test::AbsQuery(jquery) => jquery.process(state.shift_to_root()),
            Test::Function(tf) => tf.process(state),
        }
    }
}
