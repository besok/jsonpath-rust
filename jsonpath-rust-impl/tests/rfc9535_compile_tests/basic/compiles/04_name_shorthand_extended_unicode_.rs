//! Test case: 4_name_shorthand_extended_unicode_
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query! ( $.☺ ),
        Main::try_from_pest_parse("$.☺").expect("failed to parse")
    );
}
