//! Test case: 17_selector_leading_comma
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query!( $[,0] ),
        Main::try_from_pest_parse("$[,0]").expect("failed to parse")
    );
}
