//! Test case: 20_multiple_selectors_name_and_index_object_data
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        // not allowed
        // json_query! ( $['a',1] ),
        json_query!( $["a",1] ),
        Main::try_from_pest_parse("$['a',1]").expect("failed to parse")
    );
}
