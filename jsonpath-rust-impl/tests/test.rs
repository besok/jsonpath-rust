#[cfg(test)]
mod tests {
    use jsonpath_ast::ast::Main;
    use jsonpath_rust_impl::json_query;

    #[test]
    fn syn_and_pest_are_equal() {
        let q1 = (
            json_query! ( $[?@.thing > 4] ),
            Main::try_from_pest_parse("$[?@.thing > 4]").expect("failed to parse"),
        );

        assert_eq!(
            json_query! ( $[?@.thing > 4] ),
            Main::try_from_pest_parse("$[?@.thing > 4]").expect("failed to parse")
        );

        // let q2: Main = Main::try_from_pest_parse("$[?@.thing >= 5, ?@.thing <= 6]").expect("failed to parse");
        // let q3: Main = Main::try_from_pest_parse("$[?@.thing >= 5, ?@.thing <= 6.0]").expect("failed to parse");
        // let q4: Main = Main::try_from_pest_parse("$[?@.thing >= 5, ?@.thing == true]").expect("failed to parse");
        // let q5: Main = Main::try_from_pest_parse("$[?@.thing >= 5, ?@.thing != null]").expect("failed to parse");

        // let q1: Main = json_query!($[?@.thing >= 5]);
        // let q2: Main = Main::try_from_pest_parse("$[?@.thing >= 5]").expect("failed to parse");

        assert_eq!(q1.0, q1.1);
    }

    #[test]
    fn scratch() {
        let _ = json_query! ($[0,2]);
        let _ = Main::try_from_pest_parse("$[0,             2]").expect("failed to parse");
        let _ = Main::try_from_pest_parse(" $[0,2]").expect_err("failed to parse");
        let _ = Main::try_from_pest_parse("$[0,2] ").expect_err("failed to parse");
    }

    /// Common function to run trybuild for all in suite dir
    fn trybuild(dir: &str) {
        let t = ::trybuild::TestCases::new();
        let pass_path = format!("tests/rfc9535_compile_tests/{}/compiles/*.rs", dir);
        let fail_path = format!("tests/rfc9535_compile_tests/{}/fails/*.rs", dir);
        t.pass(pass_path);
        t.compile_fail(fail_path);
    }

    #[test]
    fn test_rfc_case_basic() {
        trybuild("basic");
    }
}
