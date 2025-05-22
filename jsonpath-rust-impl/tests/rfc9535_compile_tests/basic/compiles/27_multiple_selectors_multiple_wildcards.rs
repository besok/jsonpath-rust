//! Test case: 27_multiple_selectors_multiple_wildcards
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query! ( $[*,*] ),
        Main::try_from_pest_parse("$[*,*]").expect("failed to parse")
    );
}
