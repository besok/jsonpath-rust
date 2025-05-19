//! Test case: 38_descendant_segment_object_traversal_multiple_selectors
//! Tags: No tags

use ::jsonpath_ast::ast::Main;
use ::jsonpath_rust_impl::json_query;

fn main() {
    assert_eq!(
        json_query! ( $..["a","d"] ),
        Main::try_from_pest_parse("$..['a','d']").expect("failed to parse")
    );
}
