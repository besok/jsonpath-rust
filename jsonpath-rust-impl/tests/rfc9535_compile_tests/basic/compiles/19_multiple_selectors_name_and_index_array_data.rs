//! Test case: 19_multiple_selectors_name_and_index_array_data
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    // We do not accept single quotes in this household
    // assert_eq!(
    //     json_query! ( $['a',1] ),
    //     Main::try_from_pest_parse("$['a',1]").expect("failed to parse")
    // );
    assert_eq!(
        json_query! ( $["a",1] ),
        Main::try_from_pest_parse("$['a',1]").expect("failed to parse")
    );
    assert_eq!(
        json_query! ( $["a",1] ),
        Main::try_from_pest_parse("$[\"a\",1]").expect("failed to parse")
    );
}
