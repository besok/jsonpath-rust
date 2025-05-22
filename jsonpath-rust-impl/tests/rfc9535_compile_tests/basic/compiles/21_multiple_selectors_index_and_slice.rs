//! Test case: 21_multiple_selectors_index_and_slice
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query! ( $[1,5:7] ),
        Main::try_from_pest_parse("$[1,5:7]").expect("failed to parse")
    );
}
