pub mod parse {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "../../jsonpath-rust/src/parser/grammar/json_path_9535.pest"]
    pub struct JSPathParser;
}

pub mod ast {
    use super::parse::{JSPathParser, Rule};
    use derive_syn_parse::Parse;
    use from_pest::{ConversionError, FromPest, Void};
    use pest::iterators::Pairs;
    use pest::Parser;
    use pest_ast::FromPest;
    use syn::parse::{Parse, ParseStream};
    use syn::{token, LitStr, Token};

    /// Allows for syn to parse things that pest checks but does not store as rules
    #[derive(Debug)]
    pub struct PestLiteralWithoutRule<T: Default>(T);
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

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::root))]
    pub struct Root;
    impl Parse for Root {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            input.parse::<Token![$]>()?.map(|_| Root);
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::segments))]
    pub struct Segments {
        #[call(Segment::parse_outer)]
        pub segments: Vec<Segment>,
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::segment))]
    pub enum Segment {
        #[peek_with(ChildSegment::peek_any_child_segment, name = "ChildSegment")]
        Child(ChildSegment),
        #[peek(Token![..], name = "DescendantSegment")]
        Descendant(DescendantSegment),
    }

    impl Segment {
        pub fn parse_outer(input: ParseStream) -> Result<Vec<Segment>, syn::Error> {
            let mut segments = Vec::new();
            while input.peek(token::Bracket) || input.peek(Token![.]) || input.peek(Token![..]) {
                segments.push(input.parse()?);
            }
            Ok(segments)
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::child_segment))]
    pub enum ChildSegment {
        #[peek(token::Bracket, name = "BracketedSelection")]
        Bracketed(BracketedSelection),
        // search for `[` or `.`(must NOT be `..` because that is a descendant segment but syn will parse that as `..` not 2 periods)
        #[peek(Token![.], name = "WildcardSelectorOrMemberNameShorthand")]
        WildcardOrShorthand(
            PestLiteralWithoutRule<token::Dot>,
            WildcardSelectorOrMemberNameShorthand,
        ),
    }

    impl ChildSegment {
        pub fn peek_any_child_segment(input: ParseStream) -> bool {
            input.peek(token::Bracket) || input.peek(Token![.])
        }
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::descendant_segment))]
    pub struct BracketedSelection {
        #[paren]
        arg_paren: token::Bracket,
        #[inside(arg_paren)]
        selectors: Vec<Selector>,
    }
    #[derive(Debug, Parse)]
    pub enum WildcardSelectorOrMemberNameShorthand {
        #[peek(Token![*], name = "WildcardSelector")]
        WildcardSelector(WildcardSelector),
        #[peek(syn::Ident, name = "MemberNameShorthand")]
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

    #[derive(Debug, Parse)]
    struct MemberNameShorthand {
        #[call(validate_member_name_shorthand)]
        name: String,
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
    pub struct JSString(#[call(validate_js_str)] String);

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
        #[peek(token::Bracket, name = "BracketedSelection")]
        BracketedSelection(BracketedSelection),
        #[peek(Token![*], name = "WildcardSelector")]
        WildCardSelector(WildcardSelector),
        #[peek(syn::Ident, name = "MemberNameShorthand")]
        MemberNameShorthand(MemberNameShorthand),
    }

    #[derive(Debug, FromPest, Parse)]
    #[pest_ast(rule(Rule::selector))]
    pub enum Selector {
        Name(JSString),
        #[peek(Token![*], name = "WildcardSelector")]
        WildcardSelector(WildcardSelector),
        #[peek_with(|i| i.peek(Token![:]) || i.peek2(Token![:]), name = "SliceSelector")]
        SliceSelector(Option<JSInt>, Option<JSInt>, Option<JSInt>),
        #[peek_with(|i| i.peek(syn::LitInt) && !i.peek2(Token![:]), name = "IndexSelector")]
        IndexSelector(JSInt),
        FilterSelector(),
    }

    const MAX_VAL: i64 = 9007199254740991; // Maximum safe integer value in JavaScript
    const MIN_VAL: i64 = -9007199254740991; // Minimum safe integer value in JavaScript

    #[derive(Debug, Parse)]
    pub struct JSInt(#[call(validate_js_int)] i64);
    impl<'pest> FromPest for JSInt {
        type Rule = Rule;
        type FatalError = Void;

        fn from_pest(pest: &mut Pairs<'pest, Self::Rule>) -> Result<Self, ConversionError<Self::FatalError>> {
            let mut clone = pest.clone();
            let pair = clone.next().ok_or(ConversionError::NoMatch)?;
            if pair.as_rule() == Rule::member_name_shorthand {
                Ok(JSInt(common_bound_validate(pair.as_str().parse::<i64>().unwrap()).map_err(|e| ConversionError::Extraneous {})?))
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
    /// Used by both syn and pest
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
}
