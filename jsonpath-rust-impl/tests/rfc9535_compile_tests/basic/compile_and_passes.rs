// Test case: 00_root
// Tags: No tags
#[test]
fn test_00_root() {
    let q_ast = ::jsonpath_rust_impl::json_query!($);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 03_name_shorthand
// Tags: No tags
#[test]
fn test_03_name_shorthand() {
    let q_ast = ::jsonpath_rust_impl::json_query!($.a);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.a"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 04_name_shorthand_extended_unicode_uc
// Tags: No tags
#[test]
fn test_04_name_shorthand_extended_unicode_uc() {
    let q_ast = ::jsonpath_rust_impl::json_query!($["☺"]);
    #[allow(unused_variables)]
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.☺"#).expect("failed to parse");
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$["☺"]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 05_name_shorthand_underscore
// Tags: No tags
#[test]
fn test_05_name_shorthand_underscore() {
    let q_ast = ::jsonpath_rust_impl::json_query!($._);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$._"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 08_name_shorthand_absent_data
// Tags: No tags
#[test]
fn test_08_name_shorthand_absent_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($.c);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.c"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 09_name_shorthand_array_data
// Tags: No tags
#[test]
fn test_09_name_shorthand_array_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($.a);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.a"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 10_name_shorthand_object_data_nested
// Tags: No tags
#[test]
fn test_10_name_shorthand_object_data_nested() {
    let q_ast = ::jsonpath_rust_impl::json_query!($.a.b.c);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.a.b.c"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 11_wildcard_shorthand_object_data
// Tags: No tags
#[test]
fn test_11_wildcard_shorthand_object_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($.*);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.*"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 12_wildcard_shorthand_array_data
// Tags: No tags
#[test]
fn test_12_wildcard_shorthand_array_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($.*);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.*"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 13_wildcard_selector_array_data
// Tags: No tags
#[test]
fn test_13_wildcard_selector_array_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[*]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[*]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 14_wildcard_shorthand_then_name_shorthand
// Tags: No tags
#[test]
fn test_14_wildcard_shorthand_then_name_shorthand() {
    let q_ast = ::jsonpath_rust_impl::json_query!($.*.a);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$.*.a"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 15_multiple_selectors
// Tags: No tags
#[test]
fn test_15_multiple_selectors() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[0,2]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[0,2]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 19_multiple_selectors_name_and_index_array_data(EDITED: due to macro limitations)
// Tags: No tags
#[test]
fn test_19_multiple_selectors_name_and_index_array_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($["a",1]);
    let q_pest_single = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$['a',1]"#).expect("failed to parse");
    let q_pest_double = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$["a",1]"#).expect("failed to parse");
    assert_eq!(q_pest_single, q_ast);
    assert_eq!(q_pest_single, q_pest_double);
}

// Test case: 20_multiple_selectors_name_and_index_object_data(EDITED: due to macro limitations)
// Tags: No tags
#[test]
fn test_20_multiple_selectors_name_and_index_object_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($["a",1]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$['a',1]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 21_multiple_selectors_index_and_slice
// Tags: No tags
#[test]
fn test_21_multiple_selectors_index_and_slice() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[1,5:7]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[1,5:7]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 22_multiple_selectors_index_and_slice_overlapping
// Tags: No tags
#[test]
fn test_22_multiple_selectors_index_and_slice_overlapping() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[1,0:3]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[1,0:3]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 23_multiple_selectors_duplicate_index
// Tags: No tags
#[test]
fn test_23_multiple_selectors_duplicate_index() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[1,1]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[1,1]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 24_multiple_selectors_wildcard_and_index
// Tags: No tags
#[test]
fn test_24_multiple_selectors_wildcard_and_index() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[*,1]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[*,1]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 25_multiple_selectors_wildcard_and_name(EDITED: due to macro limitations)
// Tags: No tags
#[test]
fn test_25_multiple_selectors_wildcard_and_name() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[*,"a"]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[*,'a']"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 26_multiple_selectors_wildcard_and_slice
// Tags: No tags
#[test]
fn test_26_multiple_selectors_wildcard_and_slice() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[*,0:2]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[*,0:2]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 27_multiple_selectors_multiple_wildcards
// Tags: No tags
#[test]
fn test_27_multiple_selectors_multiple_wildcards() {
    let q_ast = ::jsonpath_rust_impl::json_query!($[*,*]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$[*,*]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 29_descendant_segment_index
// Tags: No tags
#[test]
fn test_29_descendant_segment_index() {
    let q_ast = ::jsonpath_rust_impl::json_query!($..[1]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$..[1]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 30_descendant_segment_name_shorthand
// Tags: No tags
#[test]
fn test_30_descendant_segment_name_shorthand() {
    let q_ast = ::jsonpath_rust_impl::json_query!($..a);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$..a"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 31_descendant_segment_wildcard_shorthand_array_data
// Tags: No tags
#[test]
fn test_31_descendant_segment_wildcard_shorthand_array_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($..*);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$..*"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 32_descendant_segment_wildcard_selector_array_data
// Tags: No tags
#[test]
fn test_32_descendant_segment_wildcard_selector_array_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($..[*]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$..[*]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 33_descendant_segment_wildcard_selector_nested_arrays
// Tags: No tags
#[test]
fn test_33_descendant_segment_wildcard_selector_nested_arrays() {
    let q_ast = ::jsonpath_rust_impl::json_query!($..[*]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$..[*]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 34_descendant_segment_wildcard_selector_nested_objects
// Tags: No tags
#[test]
fn test_34_descendant_segment_wildcard_selector_nested_objects() {
    let q_ast = ::jsonpath_rust_impl::json_query!($..[*]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$..[*]"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 35_descendant_segment_wildcard_shorthand_object_data
// Tags: No tags
#[test]
fn test_35_descendant_segment_wildcard_shorthand_object_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($..*);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$..*"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 36_descendant_segment_wildcard_shorthand_nested_data
// Tags: No tags
#[test]
fn test_36_descendant_segment_wildcard_shorthand_nested_data() {
    let q_ast = ::jsonpath_rust_impl::json_query!($..*);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$..*"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 37_descendant_segment_multiple_selectors(EDITED: due to macro limitations)
// Tags: No tags
#[test]
fn test_37_descendant_segment_multiple_selectors() {
    let q_ast = ::jsonpath_rust_impl::json_query!($..["a","d"]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$..['a','d']"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}

// Test case: 38_descendant_segment_object_traversal_multiple_selectors(EDITED: due to macro limitations)
// Tags: No tags
#[test]
fn test_38_descendant_segment_object_traversal_multiple_selectors() {
    let q_ast = ::jsonpath_rust_impl::json_query!($..["a","d"]);
    let q_pest = ::jsonpath_ast::ast::Main::try_from_pest_parse(r#"$..['a','d']"#).expect("failed to parse");
    assert_eq!(q_pest, q_ast);
}
