// Test case: 01_no_leading_whitespace(DISABLED: due to macro limitations)
// Tags: whitespace
// fn test_01_no_leading_whitespace() {
//     ::jsonpath_rust_impl::json_query!( $);
// }

// Test case: 02_no_trailing_whitespace(DISABLED: due to macro limitations)
// Tags: whitespace
// fn test_02_no_trailing_whitespace() {
//     ::jsonpath_rust_impl::json_query!($ );
// }

// Test case: 06_name_shorthand_symbol
// Tags: No tags
fn test_06_name_shorthand_symbol() {
    ::jsonpath_rust_impl::json_query!($.&);
}

// Test case: 07_name_shorthand_number
// Tags: No tags
fn test_07_name_shorthand_number() {
    ::jsonpath_rust_impl::json_query!($.1);
}

// Test case: 16_multiple_selectors_space_instead_of_comma
// Tags: whitespace
fn test_16_multiple_selectors_space_instead_of_comma() {
    ::jsonpath_rust_impl::json_query!($[0 2]);
}

// Test case: 17_selector_leading_comma
// Tags: No tags
fn test_17_selector_leading_comma() {
    ::jsonpath_rust_impl::json_query!($[,0]);
}

// Test case: 18_selector_trailing_comma
// Tags: No tags
fn test_18_selector_trailing_comma() {
    ::jsonpath_rust_impl::json_query!($[0,]);
}

// Test case: 28_empty_segment
// Tags: No tags
fn test_28_empty_segment() {
    ::jsonpath_rust_impl::json_query!($[]);
}

// Test case: 39_bald_descendant_segment
// Tags: No tags
fn test_39_bald_descendant_segment() {
    ::jsonpath_rust_impl::json_query!($..);
}

// Test case: 40_current_node_identifier_without_filter_selector
// Tags: No tags
fn test_40_current_node_identifier_without_filter_selector() {
    ::jsonpath_rust_impl::json_query!($[@.a]);
}

// Test case: 41_root_node_identifier_in_brackets_without_filter_selector
// Tags: No tags
fn test_41_root_node_identifier_in_brackets_without_filter_selector() {
    ::jsonpath_rust_impl::json_query!($[$.a]);
}

fn main() {}
