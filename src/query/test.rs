use crate::parser::model::Test;
use crate::query::context::JsonPathEvalContext;
use crate::query::queryable::Queryable;
use crate::query::state::State;
use crate::query::Query;

impl Query for Test {
    fn process_with_context<'a, T: Queryable>(
        &self,
        state: State<'a, T>,
        ctx: &mut JsonPathEvalContext,
    ) -> State<'a, T> {
        match self {
            Test::RelQuery(segments) => segments.process_with_context(state, ctx),
            Test::AbsQuery(jquery) => jquery.process_with_context(state.shift_to_root(), ctx),
            Test::Function(tf) => tf.process_with_context(state, ctx),
        }
    }
}
