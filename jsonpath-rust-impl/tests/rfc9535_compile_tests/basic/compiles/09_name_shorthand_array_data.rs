//! Test case: 9_name_shorthand_array_data
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query! ( $.a ),
        Main::try_from_pest_parse("$.a").expect("failed to parse")
    );
}
