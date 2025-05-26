//! Test case: 5_name_shorthand_underscore
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query!( $._ ),
        Main::try_from_pest_parse("$._").expect("failed to parse")
    );
}
