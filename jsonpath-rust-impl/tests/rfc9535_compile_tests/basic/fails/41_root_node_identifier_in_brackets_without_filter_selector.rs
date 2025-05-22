//! Test case: 41_root_node_identifier_in_brackets_without_filter_selector
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query! ( $[$.a] ),
        Main::try_from_pest_parse("$[$.a]").expect("failed to parse")
    );
}
