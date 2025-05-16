
#[cfg(test)]
mod tests {
    use jsonpath_ast::ast::Main;
    use jsonpath_rust_impl::json_query;

    #[test]
    fn syn_and_pest_are_equal() {
        let q1 = json_query! ( $[?@.thing > 5] );

        // let q2: Main = Main::try_from_pest_parse("$[?@.thing >= 5, ?@.thing <= 6]").expect("failed to parse");
        // let q3: Main = Main::try_from_pest_parse("$[?@.thing >= 5, ?@.thing <= 6.0]").expect("failed to parse");
        // let q4: Main = Main::try_from_pest_parse("$[?@.thing >= 5, ?@.thing == true]").expect("failed to parse");
        // let q5: Main = Main::try_from_pest_parse("$[?@.thing >= 5, ?@.thing != null]").expect("failed to parse");

        // let q1: Main = json_query!($[?@.thing >= 5]);
        // let q2: Main = Main::try_from_pest_parse("$[?@.thing >= 5]").expect("failed to parse");

        //assert_eq!(q1, q2);
    }
}