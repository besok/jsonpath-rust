pub mod parse {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "../../jsonpath-rust/src/parser/grammar/json_path_9535.pest"]
    pub struct JSPathParser;
}


mod kw {
    // syn::custom_keyword!(in);
    syn::custom_keyword!(nin);
    syn::custom_keyword!(size);
    syn::custom_keyword!(noneOf);
    syn::custom_keyword!(anyOf);
    syn::custom_keyword!(subsetOf);

    syn::custom_keyword!(null);
}

pub mod ast {
    use super::parse::{JSPathParser, Rule};
    use syn_derive::Parse;
    use from_pest::{ConversionError, FromPest, Void};
    use pest::iterators::Pairs;
    use pest::{Parser};
    use pest_ast::FromPest;
    use proc_macro2::Ident;
    use syn::parse::{Parse, ParseStream};
    use syn::punctuated::Punctuated;
    use syn::{token, LitBool, LitInt, LitStr, Token};
    use crate::parse::kw;

    pub trait KnowsRule {const RULE: Rule; }
    #[derive(Debug)]
    pub struct PestIgnoredPunctuated<T, P>(Punctuated<T, P>);
    impl<T: Parse, P: Parse> PestIgnoredPunctuated<T, P> {
        fn parse_terminated(input: ParseStream) -> syn::Result<Self> {
            Ok(PestIgnoredPunctuated(Punctuated::parse_terminated(input)?))
        }
    }
    impl<T: Parse, P: Parse> Parse for PestIgnoredPunctuated<T, P>{
        fn parse(input: ParseStream) -> syn::Result<Self> {
            Self::parse_terminated(input)
        }
    }
    impl<'pest, T, P> FromPest<'pest> for PestIgnoredPunctuated<T, P>

    where
        T: FromPest<'pest, Rule = Rule, FatalError = Void> + KnowsRule,
        P: Default
    {
        type Rule = Rule;
        type FatalError = Void;

        fn from_pest(pest: &mut Pairs<'pest, Self::Rule>) -> Result<Self, ConversionError<Self::FatalError>> {
            let mut parsed_items = Vec::new();

            while let Some(inner_pair) = pest.next() {
                if inner_pair.as_rule() == T::RULE {
                    parsed_items.push(T::from_pest( & mut inner_pair.into_inner()) ? );
                } else {
                    break;
                }
            }

            Ok(PestIgnoredPunctuated(Punctuated::from_iter(parsed_items)))
        }
    }

    /// Allows for syn to parse things that pest checks but does not store as rules
    #[derive(Debug)]
    pub struct PestLiteralWithoutRule<T>(T);
    impl<T> From<T> for PestLiteralWithoutRule<T> {
        fn from(value: T) -> Self {
            Self(value)
        }
    }
    impl<'pest, T: Default> FromPest<'pest> for PestLiteralWithoutRule<T> {
        type Rule = Rule;
        type FatalError = Void;

        /// Always generates default value and leaves parse stream alone
        fn from_pest(
            _pest: &mut Pairs<'pest, Self::Rule>,
        ) -> Result<Self, ConversionError<Self::FatalError>> {
            Ok(PestLiteralWithoutRule(T::default()))
        }
    }

    impl<T: Default + Parse> Parse for PestLiteralWithoutRule<T> {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            Ok(PestLiteralWithoutRule(input.parse::<T>()?))
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::main))]
    pub struct Main {
        pub jp_query: JPQuery,
        pub eoi: EOI,
    }

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::EOI))]
    pub struct EOI;
    impl Parse for EOI {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            if input.is_empty() {
                Ok(Self)
            } else {
                Err(input.error("Unexpected token"))
            }
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::jp_query))]
    pub struct JPQuery {
        pub root: Root,
        pub segments: Segments,
    }

    impl JPQuery {
        fn peek(input: ParseStream) -> bool {
            Root::peek(input)
        }
    }

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::root))]
    pub struct Root;
    impl Parse for Root {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let _ = input.parse::<Token![$]>()?;
            Ok(Root)
        }
    }

    impl Root {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![$])
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::segments))]
    pub struct Segments {
        #[parse(Segment::parse_outer)]
        pub segments: Vec<Segment>,
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::segment))]
    pub enum Segment {
        #[parse(peek_func = ChildSegment::peek)]
        Child(ChildSegment),
        #[parse(peek_func = DescendantSegment::peek)]
        Descendant(DescendantSegment),
    }

    impl Segment {
        fn peek(input: ParseStream) -> bool {
            ChildSegment::peek(input) || DescendantSegment::peek(input)
        }

        pub fn parse_outer(input: ParseStream) -> Result<Vec<Segment>, syn::Error> {
            let mut segments = Vec::new();
            while Self::peek(input) {
                segments.push(input.parse()?);
            }
            Ok(segments)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::child_segment))]
    pub enum ChildSegment {
        #[parse(peek_func = BracketedSelection::peek)]
        Bracketed(BracketedSelection),
        // search for `[` or `.`(must NOT be `..` because that is a descendant segment but syn will parse that as `..` not 2 periods)
        #[parse(peek = Token![.])]
        WildcardOrShorthand(
            PestLiteralWithoutRule<token::Dot>,
            WildcardSelectorOrMemberNameShorthand,
        ),
    }

    impl ChildSegment {
        fn peek(input: ParseStream) -> bool {
            input.peek(token::Bracket) || input.peek(Token![.])
        }
    }

    #[derive(Debug, Parse)]
    pub struct BracketedSelection {
        #[syn(bracketed)]
        arg_bracket: token::Bracket,
        #[syn(in = arg_bracket)]
        #[parse(|i: ParseStream| PestIgnoredPunctuated::parse_terminated(i))]
        selectors: PestIgnoredPunctuated<Selector, Token![,]>,
    }

    impl BracketedSelection {
        fn peek(input: ParseStream) -> bool {
            input.peek(token::Bracket)
        }
    }

    impl<'pest> from_pest::FromPest<'pest> for BracketedSelection {
        type Rule = Rule;
        type FatalError = Void;
        fn from_pest(pest: &mut Pairs<'pest, Rule>) -> Result<Self, ConversionError<Void>> {
            let mut clone = pest.clone();
            let pair = clone.next().ok_or(ConversionError::NoMatch)?;
            if pair.as_rule() == Rule::bracketed_selection {
                let mut inner = pair.into_inner();
                let inner = &mut inner;
                let this = BracketedSelection {
                    arg_bracket: Default::default(),
                    selectors: FromPest::from_pest(inner)?,
                };
                if inner.clone().next().is_some() {
                    Err(ConversionError::Extraneous {
                        current_node: "BracketedSelection",
                    })?;
                }
                *pest = clone;
                Ok(this)
            } else {
                Err(ConversionError::NoMatch)
            }
        }
    }

    #[derive(Debug, Parse)]
    pub enum WildcardSelectorOrMemberNameShorthand {
        #[parse(peek_func = WildcardSelector::peek)]
        WildcardSelector(WildcardSelector),
        #[parse(peek_func = MemberNameShorthand::peek)]
        MemberNameShorthand(MemberNameShorthand),
    }
    impl<'pest> FromPest<'pest> for WildcardSelectorOrMemberNameShorthand {
        type Rule = Rule;
        type FatalError = Void;

        fn from_pest(
            pest: &mut Pairs<'pest, Self::Rule>,
        ) -> Result<Self, ConversionError<Self::FatalError>> {
            // let _ = pest.as_str().strip_prefix(".");
            let mut clone = pest.clone();
            let pair = clone.next().ok_or(ConversionError::NoMatch)?;

            match pair.as_rule() {
                Rule::wildcard_selector => {
                    let mut inner = pair.clone().into_inner();
                    let inner = &mut inner;
                    let this = Self::WildcardSelector(::from_pest::FromPest::from_pest(inner)?);
                    // if inner.clone().next().is_some() {
                    //     ::from_pest::log::trace!(
                    //         "when converting {}, found extraneous {:?}",
                    //         stringify!(ChildSegment),
                    //         stringify!(Bracketed)
                    //     );
                    //     Err(ConversionError::Extraneous {
                    //         current_node: stringify!(Bracketed),
                    //     })?;
                    // }
                    Ok(this)
                }
                Rule::member_name_shorthand => {
                    let mut inner = pair.clone().into_inner();
                    let inner = &mut inner;
                    let this = Self::MemberNameShorthand(::from_pest::FromPest::from_pest(inner)?);
                    // if inner.clone().next().is_some() {
                    //     ::from_pest::log::trace!(
                    //         "when converting {}, found extraneous {:?}",
                    //         stringify!(ChildSegment),
                    //         stringify!(Bracketed)
                    //     );
                    //     Err(ConversionError::Extraneous {
                    //         current_node: stringify!(Bracketed),
                    //     })?;
                    // }
                    Ok(this)
                }
                _ => Err(ConversionError::NoMatch),
            }
        }
    }

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::wildcard_selector))]
    pub struct WildcardSelector;
    /// A named rule exists for this, so it's easier to let the FromPest automatically generate and
    ///     just harvest the wildcard token manually in syn
    impl Parse for WildcardSelector {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            input.parse::<Token![*]>().map(|_| WildcardSelector)
        }
    }
    impl WildcardSelector {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![*])
        }
    }

    #[derive(Debug, Parse)]
    struct MemberNameShorthand {
        #[parse(validate_member_name_shorthand)]
        name: String,
    }

    impl MemberNameShorthand {
        fn peek(input: ParseStream) -> bool {
            input.peek(syn::Ident)
        }
    }

    fn validate_member_name_shorthand(input: ParseStream) -> Result<String, syn::Error> {
        let ident = input.parse::<syn::Ident>()?;
        match JSPathParser::parse(Rule::member_name_shorthand, &ident.to_string()) {
            Ok(_) => Ok(ident.to_string()),
            Err(e) => Err(syn::Error::new(ident.span(), e.to_string())),
        }
    }

    impl<'pest> from_pest::FromPest<'pest> for MemberNameShorthand {
        type Rule = Rule;
        type FatalError = Void;
        fn from_pest(pest: &mut Pairs<'pest, Rule>) -> Result<Self, ConversionError<Void>> {
            let mut clone = pest.clone();
            let pair = clone.next().ok_or(ConversionError::NoMatch)?;
            if pair.as_rule() == Rule::member_name_shorthand {
                Ok(MemberNameShorthand {
                    name: pair.as_str().to_string(),
                })
            } else {
                Err(ConversionError::NoMatch)
            }
        }
    }

    #[derive(Debug, Parse)]
    pub struct JSString(#[parse(validate_js_str)] String);
    impl JSString {
        fn peek(input: ParseStream) -> bool {
            input.peek(syn::LitStr) || input.peek(syn::LitChar)
        }
    }

    impl<'pest> from_pest::FromPest<'pest> for JSString {
        type Rule = Rule;
        type FatalError = Void;
        fn from_pest(pest: &mut Pairs<'pest, Rule>) -> Result<Self, ConversionError<Void>> {
            let mut clone = pest.clone();
            let pair = clone.next().ok_or(ConversionError::NoMatch)?;
            if pair.as_rule() == Rule::string {
                let inner = pair.into_inner();
                let this = JSString(inner.to_string());
                if inner.clone().next().is_some() {
                    from_pest::log::trace!(
                        "when converting {}, found extraneous {:?}",
                        stringify!(JSString),
                        inner
                    );
                    Err(ConversionError::Extraneous {
                        current_node: stringify!(JSString),
                    })?;
                }
                *pest = clone;
                Ok(this)
            } else {
                Err(ConversionError::NoMatch)
            }
        }
    }

    /// Validates a JSONPath string literal according to RFC 9535
    /// Control characters (U+0000 through U+001F and U+007F) are not allowed unescaped
    /// in string literals, whether single-quoted or double-quoted
    fn validate_js_str(input: ParseStream) -> Result<String, syn::Error> {
        let lit_str = input.parse::<LitStr>()?;
        let s = lit_str.value();
        for (i, c) in s.chars().enumerate() {
            if c <= '\u{001F}' {
                return Err(syn::Error::new(
                    lit_str.span(),
                    format!(
                        "Invalid control character U+{:04X} at position {} in string literal",
                        c as u32, i
                    ),
                ));
            }
        }

        Ok(s)
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::descendant_segment))]
    pub enum DescendantSegment {
        #[parse(peek_func = BracketedSelection::peek)]
        BracketedSelection(BracketedSelection),
        #[parse(peek_func = WildcardSelector::peek)]
        WildcardSelector(WildcardSelector),
        #[parse(peek_func = MemberNameShorthand::peek)]
        MemberNameShorthand(MemberNameShorthand),
    }
    impl DescendantSegment {
        fn peek(input: ParseStream) -> bool {
            BracketedSelection::peek(input)
                || WildcardSelector::peek(input)
                || MemberNameShorthand::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::selector))]
    pub enum Selector {
        #[parse(peek_func = WildcardSelector::peek)]
        WildcardSelector(WildcardSelector),
        #[parse(peek_func = SliceSelector::peek)]
        SliceSelector(SliceSelector),
        #[parse(peek_func = JSInt::peek)]
        IndexSelector(JSInt),
        #[parse(peek_func = FilterSelector::peek)]
        FilterSelector(FilterSelector),
        // This MUST be the last element to prevent syn::Lit from catching one of the others, it's our "fallback"
        #[parse(peek_func = JSString::peek)]
        NameSelector(JSString),
    }
    impl KnowsRule for Selector { const RULE: Rule = Rule::selector; }

    impl Selector {
        pub fn peek(input: ParseStream) -> bool {
            WildcardSelector::peek(input)
                || (input.peek(Token![:]) || input.peek2(Token![:]))
                || JSInt::peek(input)
                || FilterSelector::peek(input)
                || JSString::peek(input)
        }
        pub fn parse_outer(input: ParseStream) -> Result<Vec<Selector>, syn::Error> {
            let mut selectors = Vec::new();
            while Self::peek(input) {
                selectors.push(input.parse()?);
            }
            Ok(selectors)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::slice_selector))]
    pub struct SliceSelector(
        #[parse(SliceStart::maybe_parse)]
        Option<SliceStart>,
        PestLiteralWithoutRule<Token![:]>,
        #[parse(SliceEnd::maybe_parse)]
        Option<SliceEnd>,
        #[parse(SliceStep::maybe_parse)]
        Option<SliceStep>
    );
    impl SliceSelector {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![:]) || input.peek2(Token![:])
        }
    }

    // impl Parse for SliceSelector {
    //     fn parse(input: ParseStream) -> syn::Result<Self> {
    //         let start: Option<SliceStart> = if JSInt::peek(input) {
    //             Some(input.parse()?)
    //         } else {
    //             None
    //         };
    //         let colon: Token![:] = input.parse()?;
    //         let end: Option<SliceEnd> = if input.peek(Token![:]) || input.peek2(Token![:]) {
    //             Some(input.parse()?)
    //         } else {
    //             None
    //         };
    //         let step: Option<SliceStep> = if input.peek(Token![:]) || input.peek2(LitInt) {
    //             Some(input.parse()?)
    //         } else {
    //             None
    //         };
    //         Ok(Self(start, PestLiteralWithoutRule(colon), end, step))
    //     }
    // }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::step))]
    pub struct SliceStep(PestLiteralWithoutRule<Token![:]>, JSInt);
    impl SliceStep {
        fn maybe_parse(input: ParseStream) -> syn::Result<Option<Self>> {
            if input.peek(Token![:]) {
                Ok(Some(input.parse()?))
            } else {
                Ok(None)
            }
        }

    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::start))]
    pub struct SliceStart(JSInt);
    impl SliceStart {
        fn maybe_parse(input: ParseStream) -> syn::Result<Option<Self>> {
            if input.peek(Token![:]) {
                Ok(Some(input.parse()?))
            } else {
                Ok(None)
            }
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::end))]
    pub struct SliceEnd(JSInt);
    impl SliceEnd {
        fn maybe_parse(input: ParseStream) -> syn::Result<Option<Self>> {
            if JSInt::peek(input) {
                Ok(Some(input.parse()?))
            } else {
                Ok(None)
            }
        }
    }


    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::filter_selector))]
    pub struct FilterSelector {
        pub q: PestLiteralWithoutRule<Token![?]>,
        expr: LogicalExpr
    }

    impl FilterSelector {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![?])
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::logical_expr))]
    pub struct LogicalExpr {
        ands: PestIgnoredPunctuated<LogicalExprAnd, Token![||]>,
    }

    impl LogicalExpr {
        fn peek(input: ParseStream) -> bool {
            LogicalExprAnd::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::logical_expr_and))]
    pub struct LogicalExprAnd {
        atoms: PestIgnoredPunctuated<AtomExpr, Token![&&]>,
    }
    impl KnowsRule for LogicalExprAnd { const RULE: Rule = Rule::logical_expr_and; }

    impl LogicalExprAnd {
        fn peek(input: ParseStream) -> bool {
            AtomExpr::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::atom_expr))]
    pub enum AtomExpr {
        #[parse(peek_func = ParenExpr::peek)]
       ParenExpr(ParenExpr),
        #[parse(peek_func = CompExpr::peek)]
       CompExpr(CompExpr),
        #[parse(peek_func = TestExpr::peek)]
       TestExpr(TestExpr)
    }
    impl KnowsRule for AtomExpr { const RULE: Rule = Rule::atom_expr; }

    impl AtomExpr {
        fn peek(input: ParseStream) -> bool {
            ParenExpr::peek(input) || CompExpr::peek(input) || TestExpr::peek(input)
        }
    }


    const MAX_VAL: i64 = 9007199254740991; // Maximum safe integer value in JavaScript
    const MIN_VAL: i64 = -9007199254740991; // Minimum safe integer value in JavaScript

    #[derive(Debug, Parse)]
    pub struct JSInt(#[parse(validate_js_int)] i64);

    impl JSInt {
        fn peek(input: ParseStream) -> bool {
            input.peek(LitInt)
        }
    }
    impl<'pest> FromPest<'pest> for JSInt {
        type Rule = Rule;
        type FatalError = Void;

        fn from_pest(
            pest: &mut Pairs<'pest, Self::Rule>,
        ) -> Result<Self, ConversionError<Self::FatalError>> {
            let mut clone = pest.clone();
            let pair = clone.next().ok_or(ConversionError::NoMatch)?;
            if pair.as_rule() == Rule::int {
                Ok(JSInt(
                    pair.as_str()
                        .parse::<i64>()
                        .expect("int rule should always be a valid i64"),
                ))
            } else {
                Err(ConversionError::NoMatch)
            }
        }
    }

    /// Only used by syn
    fn validate_js_int(input: ParseStream) -> Result<i64, syn::Error> {
        let lit_int = input.parse::<syn::LitInt>()?;
        let parsed = lit_int.base10_parse::<i64>()?;
        Ok(common_bound_validate(parsed).map_err(|e| syn::Error::new(lit_int.span(), e))?)
    }
    /// Used by both syn ~~and pest~~(pest changed to use range constraints)
    fn common_bound_validate(num: i64) -> Result<i64, String> {
        if num > MAX_VAL || num < MIN_VAL {
            let info = if num > MAX_VAL {
                ("greater", "maximum", MAX_VAL)
            } else {
                ("less", "minimum", MIN_VAL)
            };
            return Err(format!(
                "number out of bounds: {} is {} than {} JS integer value: {}",
                num, info.0, info.1, info.2,
            ));
        }
        Ok(num)
    }

    // New implementations below LLM STUFF

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::paren_expr))]
    pub struct ParenExpr {
        // #[parse(peek_func = NotOp::peek)]
        not_op: Option<NotOp>,
        // #[paren]
        paren: PestLiteralWithoutRule<token::Paren>,
        // #[inside(paren)]
        expr: LogicalExpr,
    }

    impl Parse for ParenExpr {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let not_op: Option<NotOp> = if NotOp::peek(input) {
                Some(input.parse()?)
            } else { None };
            let __paren_backing_token_stream;
            let paren: PestLiteralWithoutRule<token::Paren> = syn::parenthesized!(__paren_backing_token_stream in input ).into();
            let expr: LogicalExpr = __paren_backing_token_stream.parse()?;
            Ok(ParenExpr {
                not_op,
                paren,
                expr,
            })
        }
    }

    impl ParenExpr {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![!]) || input.peek(token::Paren)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::comp_expr))]
    pub struct CompExpr {
        left: Comparable,
        op: CompOp,
        right: Comparable,
    }

    impl CompExpr {
        fn peek(input: ParseStream) -> bool {
            Comparable::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::test_expr))]
    pub struct TestExpr {
        #[parse(NotOp::maybe_parse)]
        not_op: Option<NotOp>,
        test: Test,
    }

    impl TestExpr {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![!]) || Test::peek(input)
        }
    }

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::not_op))]
    pub struct NotOp;

    impl Parse for NotOp {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            input.parse::<Token![!]>().map(|_| NotOp)
        }
    }

    impl NotOp {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![!])
        }

        fn maybe_parse(input: ParseStream) -> syn::Result<Option<Self>> {
            Ok(if Self::peek(input) {
                Some(input.parse()?)
            } else {
                None
            })
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::test))]
    pub enum Test {
        #[parse(peek_func = RelQuery::peek)]
        RelQuery(RelQuery),
        #[parse(peek_func = JPQuery::peek)]
        JPQuery(JPQuery),
        #[parse(peek_func = FunctionExpr::peek)]
        FunctionExpr(FunctionExpr),
    }

    impl Test {
        fn peek(input: ParseStream) -> bool {
            RelQuery::peek(input) || JPQuery::peek(input) || FunctionExpr::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::comparable))]
    pub enum Comparable {
        #[parse(peek_func = Literal::peek)]
        Literal(Literal),
        #[parse(peek_func = SingularQuery::peek)]
        SingularQuery(SingularQuery),
        #[parse(peek_func = FunctionExpr::peek)]
        FunctionExpr(FunctionExpr),
    }

    impl Comparable {
        fn peek(input: ParseStream) -> bool {
            Literal::peek(input) || SingularQuery::peek(input) || FunctionExpr::peek(input)
        }
    }

    #[derive(Debug, Parse)]
    pub enum CompOp {
        #[parse(peek = Token![==])]
        Eq(Token![==]),
        #[parse(peek = Token![!=])]
        Ne(Token![!=]),
        #[parse(peek = Token![<=])]
        Le(Token![<=]),
        #[parse(peek = Token![>=])]
        Ge(Token![>=]),
        #[parse(peek = Token![<])]
        Lt(Token![<]),
        #[parse(peek = Token![>])]
        Gt(Token![>]),
        // #[parse(peek_func = |input: ParseStream| input.peek(syn::token::In))]
        // In(syn::token::In),
        // #[parse(peek_func = |input: ParseStream| input.peek(kw::nin))]
        // Nin(kw::nin),
        // #[parse(peek_func = |input: ParseStream| input.peek(kw::size))]
        // Size(kw::size),
        // #[parse(peek_func = |input: ParseStream| input.peek(kw::noneOf))]
        // NoneOf(kw::noneOf),
        // #[parse(peek_func = |input: ParseStream| input.peek(kw::anyOf))]
        // AnyOf(kw::anyOf),
        // #[parse(peek_func = |input: ParseStream| input.peek(kw::subsetOf))]
        // SubsetOf(kw::subsetOf),
    }
    impl KnowsRule for CompOp { const RULE: Rule = Rule::comp_op; }
    impl<'pest> FromPest<'pest> for CompOp {
        type Rule = Rule;
        type FatalError = Void;

        fn from_pest(pest: &mut Pairs<'pest, Self::Rule>) -> Result<Self, ConversionError<Self::FatalError>> {
            let mut clone = pest.clone();
            let pair = clone.next().ok_or(ConversionError::NoMatch)?;
            if pair.as_rule() == Self::RULE {
                Ok(match pair.into_inner().to_string().trim() {
                    "==" => {Self::Eq(Default::default())},
                    "!=" => {Self::Ne(Default::default())},
                    "<=" => {Self::Le(Default::default())},
                    ">=" => {Self::Ge(Default::default())},
                    "<" => {Self::Lt(Default::default())},
                    ">" => {Self::Gt(Default::default())},
                    // "in" => {Self::In(Default::default())},
                    // "nin" => {Self::Nin(Default::default())},
                    // "size" => {Self::Size(Default::default())},
                    // "noneOf" => {Self::NoneOf(Default::default())},
                    // "anyOf" => {Self::AnyOf(Default::default())},
                    // "subsetOf" => {Self::SubsetOf(Default::default())},
                    _ => unreachable!(),
                })
            } else {
                Err(ConversionError::NoMatch)
            }
        }
    }

    #[derive(Debug, Parse)]
    pub struct FunctionExpr {
        name: FunctionName,
        #[syn(parenthesized)]
        paren: token::Paren,
        #[syn(in = paren)]
        #[parse(|i: ParseStream| PestIgnoredPunctuated::parse_terminated(i))]
        args: PestIgnoredPunctuated<FunctionArgument, Token![,]>,
    }

    impl<'pest> from_pest::FromPest<'pest> for FunctionExpr {
        type Rule = Rule;
        type FatalError = Void;
        fn from_pest(pest: &mut Pairs<'pest, Rule>) -> Result<Self, ConversionError<Void>> {
            let mut clone = pest.clone();
            let pair = clone.next().ok_or(ConversionError::NoMatch)?;
            if pair.as_rule() == Rule::function_expr {
                let span = pair.as_span();
                let mut inner = pair.into_inner();
                let inner = &mut inner;
                let this = FunctionExpr {
                    name: ::from_pest::FromPest::from_pest(inner)?,
                    paren: Default::default(),
                    args: FromPest::from_pest(inner)?,
                };
                if inner.clone().next().is_some() {
                    Err(ConversionError::Extraneous { current_node: "FunctionExpr" })?;
                }
                *pest = clone;
                Ok(this)
            } else { Err(ConversionError::NoMatch) }
        }
    }

    impl FunctionExpr {
        fn peek(input: ParseStream) -> bool {
            FunctionName::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::function_name))]
    pub struct FunctionName {
        #[parse(validate_function_name)]
        name: syn::Ident,
    }

    impl FunctionName {
        fn peek(input: ParseStream) -> bool {
            input.peek(syn::Ident)
        }
    }

    fn validate_function_name(input: ParseStream) -> Result<Ident, syn::Error> {
        let ident = input.parse::<syn::Ident>()?;
        match JSPathParser::parse(Rule::function_name, &ident.to_string()) {
            Ok(_) => Ok(ident),
            Err(e) => Err(syn::Error::new(ident.span(), e.to_string())),
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::function_argument))]
    pub enum FunctionArgument {
        #[parse(peek_func = Literal::peek)]
        Literal(Literal),
        #[parse(peek_func = Test::peek)]
        Test(Test),
        #[parse(peek_func = LogicalExpr::peek)]
        LogicalExpr(LogicalExpr),
    }
    impl KnowsRule for FunctionArgument { const RULE: Rule = Rule::function_argument; }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::rel_query))]
    pub struct RelQuery {
        curr: Curr,
        segments: Segments,
    }

    impl RelQuery {
        fn peek(input: ParseStream) -> bool {
            Curr::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::curr))]
    pub struct Curr(Token![@]);
    impl Curr {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![@])
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::singular_query))]
    pub enum SingularQuery {
        #[parse(peek_func = RelSingularQuery::peek)]
        RelSingularQuery(RelSingularQuery),
        #[parse(peek_func = AbsSingularQuery::peek)]
        AbsSingularQuery(AbsSingularQuery),
    }

    impl SingularQuery {
        fn peek(input: ParseStream) -> bool {
            RelSingularQuery::peek(input) || AbsSingularQuery::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::rel_singular_query))]
    pub struct RelSingularQuery {
        curr: Curr,
        segments: SingularQuerySegments,
    }

    impl RelSingularQuery {
        fn peek(input: ParseStream) -> bool {
            Curr::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::abs_singular_query))]
    pub struct AbsSingularQuery {
        root: Root,
        segments: SingularQuerySegments,
    }

    impl AbsSingularQuery {
        fn peek(input: ParseStream) -> bool {
            Root::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::singular_query_segments))]
    pub struct SingularQuerySegments {
        #[parse(SingularQuerySegment::parse_outer)]
        segments: Vec<SingularQuerySegment>,
    }

    #[derive(Debug, Parse)]
    pub enum SingularQuerySegment {
        #[parse(peek_func = NameSegment::peek)]
        NameSegment(NameSegment),
        #[parse(peek_func = IndexSegment::peek)]
        IndexSegment(IndexSegment),
    }

    impl SingularQuerySegment {
        fn peek(input: ParseStream) -> bool {
            NameSegment::peek(input) || IndexSegment::peek(input)
        }

        fn parse_outer(input: ParseStream) -> Result<Vec<SingularQuerySegment>, syn::Error> {
            let mut segments = Vec::new();
            while Self::peek(input) {
                segments.push(input.parse()?);
            }
            Ok(segments)
        }
    }

    impl<'pest> FromPest<'pest> for SingularQuerySegment {
        type Rule = Rule;
        type FatalError = Void;

        fn from_pest(
            pest: &mut Pairs<'pest, Self::Rule>,
        ) -> Result<Self, ConversionError<Self::FatalError>> {
            let mut clone = pest.clone();
            let pair = clone.next().ok_or(ConversionError::NoMatch)?;

            match pair.as_rule() {
                Rule::name_segment => {
                    let mut inner = pair.clone().into_inner();
                    let inner = &mut inner;
                    let this = Self::NameSegment(::from_pest::FromPest::from_pest(inner)?);
                    Ok(this)
                }
                Rule::index_segment => {
                    let mut inner = pair.clone().into_inner();
                    let inner = &mut inner;
                    let this = Self::IndexSegment(::from_pest::FromPest::from_pest(inner)?);
                    Ok(this)
                }
                _ => Err(ConversionError::NoMatch),
            }
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::name_segment))]
    pub enum NameSegment {
        #[parse(peek = token::Bracket)]
        BracketedName(
            PestLiteralWithoutRule<token::Bracket>,
            JSString,
            PestLiteralWithoutRule<token::Bracket>,

        ),
        #[parse(peek = Token![.])]
        DotName(
            PestLiteralWithoutRule<Token![.]>,
            MemberNameShorthand,
        ),
    }

    impl NameSegment {
        fn peek(input: ParseStream) -> bool {
            input.peek(token::Bracket) || input.peek(Token![.])
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::index_segment))]
    pub struct IndexSegment {
        #[syn(bracketed)]
        bracket: token::Bracket,
        #[syn(in = bracket)]
        index: JSInt,
    }

    impl IndexSegment {
        fn peek(input: ParseStream) -> bool {
            input.peek(token::Bracket)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::literal))]
    pub enum Literal {
        #[parse(peek_func = Number::peek)]
        Number(Number),
        #[parse(peek_func = JSString::peek)]
        String(JSString),
        #[parse(peek = Bool::peek)]
        Bool(bool),
        #[parse(peek_func = Null::peek)]
        Null(Null),
    }

    impl Literal {
        fn peek(input: ParseStream) -> bool {
            Number::peek(input) || JSString::peek(input) || Bool::peek(input) || Null::peek(input)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::number))]
    pub struct Number {
        #[parse(validate_number)]
        value: f64,
    }

    impl Number {
        fn peek(input: ParseStream) -> bool {
            input.peek(syn::LitInt) || input.peek(syn::LitFloat)
        }
    }

    fn validate_number(input: ParseStream) -> Result<f64, syn::Error> {
        if input.peek(syn::LitInt) {
            let lit_int = input.parse::<syn::LitInt>()?;
            let parsed = lit_int.base10_parse::<f64>()?;
            Ok(parsed)
        } else if input.peek(syn::LitFloat) {
            let lit_float = input.parse::<syn::LitFloat>()?;
            let parsed = lit_float.base10_parse::<f64>()?;
            Ok(parsed)
        } else {
            Err(input.error("Expected number"))
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::bool))]
    pub struct Bool(bool);

    impl Bool {
        fn peek(input: ParseStream) -> bool {
            input.peek(LitBool)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::null))]
    pub struct Null(kw::null);

    impl Null {
        fn peek(input: ParseStream) -> bool {
            input.peek(kw::null)
        }
    }
}
