use crate::parser::model2::Filter;
use crate::query::queryable::Queryable;
use crate::query::state::State;
use crate::query::Query;

impl Query for Filter {
    fn process<'a, T: Queryable>(&self, state: State<'a, T>) -> State<'a, T> {
        match self {
            Filter::Or(ors) => State::bool(
                ors.iter().any(|or| {
                    or.process(state.clone())
                        .val()
                        .and_then(|v| v.as_bool())
                        .unwrap_or_default()
                }),
                state.root,
            ),
            Filter::And(ands) => State::bool(
                ands.iter().all(|and| {
                    and.process(state.clone())
                        .val()
                        .and_then(|v| v.as_bool())
                        .unwrap_or_default()
                }),
                state.root,
            ),
            Filter::Atom(atom) => atom.process(state),
        }
    }
}
