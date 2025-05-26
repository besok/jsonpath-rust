//! Test case: 8_name_shorthand_absent_data
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query!( $.c ),
        Main::try_from_pest_parse("$.c").expect("failed to parse")
    );
}
