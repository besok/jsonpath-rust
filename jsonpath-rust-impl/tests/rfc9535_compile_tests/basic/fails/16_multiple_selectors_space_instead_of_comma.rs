//! Test case: 16_multiple_selectors_space_instead_of_comma
//! Tags: whitespace

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query!( $[0 2] ),
        Main::try_from_pest_parse("$[0 2]").expect("failed to parse")
    );
}
