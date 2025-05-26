//! Test case: 22_multiple_selectors_index_and_slice_overlapping
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query!( $[1,0:3] ),
        Main::try_from_pest_parse("$[1,0:3]").expect("failed to parse")
    );
}
