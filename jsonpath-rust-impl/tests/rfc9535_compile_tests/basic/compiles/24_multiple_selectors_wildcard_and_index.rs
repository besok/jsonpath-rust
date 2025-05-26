//! Test case: 24_multiple_selectors_wildcard_and_index
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query!( $[*,1] ),
        Main::try_from_pest_parse("$[*,1]").expect("failed to parse")
    );
}
