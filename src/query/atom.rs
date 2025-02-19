use crate::parser::model2::FilterAtom;
use crate::query::queryable::Queryable;
use crate::query::state::State;
use crate::query::Query;

impl Query for FilterAtom {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        match self {
            FilterAtom::Filter { expr, not } => process_with_not(expr, state, *not),
            FilterAtom::Test { expr, not } => process_with_not(expr, state, *not),
            FilterAtom::Comparison(cmp) => cmp.process(state),
        }
    }
}

fn process_with_not<'a, T, Q>(expr: &Box<Q>, state: State<'a, T>, not: bool) -> State<'a, T>
where
    Q: Query,
    T: Queryable,
{
    let root = state.root;
    let bool_res = expr.process(state);
    if not {
        bool_res
            .val()
            .and_then(|v| v.as_bool())
            .map(|bool| State::bool(!bool, root))
            .unwrap_or_else(|| State::bool(false, root))
    } else {
        bool_res
    }
}
