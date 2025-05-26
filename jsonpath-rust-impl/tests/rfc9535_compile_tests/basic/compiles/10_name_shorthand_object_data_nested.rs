//! Test case: 10_name_shorthand_object_data_nested
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query!( $.a.b.c ),
        Main::try_from_pest_parse("$.a.b.c").expect("failed to parse")
    );
}
