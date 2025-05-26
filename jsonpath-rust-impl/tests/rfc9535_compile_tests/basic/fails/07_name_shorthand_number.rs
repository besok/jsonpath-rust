//! Test case: 07_name_shorthand_number
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query!( $.1 ),
        Main::try_from_pest_parse(r#"$.1"#).expect("failed to parse")
    );
}
