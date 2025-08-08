#[cfg(feature = "compiled-path")]
pub(crate) mod parse_impl {
    use crate::ast::parse::{JSPathParser, Rule};
    use crate::ast::{kw, CompOp, FnCallOneArg, FnCallTwoArg, IndexSelector, Main, NameSelector, NodesType, ReturnsLogical, ReturnsNodes, ReturnsValue, ValueType};
    use crate::ast::{
        AbsSingularQuery, AtomExpr, Bool, BracketName, BracketedSelection, ChildSegment, CompExpr,
        Comparable, DescendantSegment, FilterSelector, FunctionExpr, IndexSegment, JPQuery, JSInt, JSString, Literal, LogicalExpr, LogicalExprAnd, MemberNameShorthand,
        NameSegment, NotOp, Null, Number, ParenExpr, PestLiteral, PestWithIgnoredPunctuation,
        RelQuery, RelSingularQuery, Root, Segment, Segments, Selector, SingularQuery,
        SingularQuerySegment, SingularQuerySegments, SliceEnd, SliceSelector, SliceStart,
        SliceStep, Test, TestExpr, WildcardSelector, WildcardSelectorOrMemberNameShorthand,
        EOI,
    };
    use pest::Parser as PestParser;
    use proc_macro2::{Ident, TokenStream};
    use quote::{quote, ToTokens};
    use syn::parse::{Parse, ParseStream};
    use syn::punctuated::Punctuated;
    use syn::spanned::Spanned;
    use syn::token::Token;
    use syn::{token, LitBool, LitInt, LitStr, Token};

    pub trait ParseUtilsExt: Parse {
        fn peek(input: ParseStream) -> bool;
        fn maybe_parse(input: ParseStream) -> syn::Result<Option<Self>> {
            Ok(if Self::peek(input) {
                Some(input.parse()?)
            } else {
                None
            })
        }

        fn parse_outer(input: ParseStream) -> Result<Vec<Self>, syn::Error> {
            let mut items = Vec::new();
            while Self::peek(input) {
                items.push(input.parse()?);
            }
            Ok(items)
        }
    }

    impl<T: Parse, P: Parse> PestWithIgnoredPunctuation<T, P> {
        pub(crate) fn parse_terminated(input: ParseStream) -> syn::Result<Self> {
            Ok(PestWithIgnoredPunctuation(Punctuated::parse_terminated(input)?))
        }

        pub(crate) fn parse_separated_nonempty(input: ParseStream) -> syn::Result<Self>
        where
            P: Token,
        {
            let res = Punctuated::parse_separated_nonempty(input)?;
            if res.is_empty() {
                Err(input.error(format!(
                    "Expected at least one {}",
                    std::any::type_name::<T>()
                )))
            } else {
                Ok(PestWithIgnoredPunctuation(res))
            }
        }
    }

    impl<T: Parse, P: Parse> Parse for PestWithIgnoredPunctuation<T, P> {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            Self::parse_terminated(input)
        }
    }

    impl<T: Default + Parse> Parse for PestLiteral<T> {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            Ok(PestLiteral(input.parse::<T>()?))
        }
    }
    impl<T: ToTokens> ToTokens for PestLiteral<T> {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self { 0: __0 } = self;
            {
                {
                    let __expr: fn(&mut TokenStream, _) = |tokens, val: &T| {
                        let mut sub = TokenStream::new();
                        val.to_tokens(&mut sub);
                        tokens.extend(
                            quote! { ::jsonpath_ast::ast::PestLiteral::new(Default::default()) },
                        );
                    };
                    __expr(tokens, __0)
                };
            }
        }
    }

    impl ToTokens for Main {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let (mut q, mut e) = (TokenStream::new(), TokenStream::new());
            self.jp_query.to_tokens(&mut q);
            self.eoi.to_tokens(&mut e);
            tokens.extend(quote! {
                ::jsonpath_ast::ast::Main::new(
                    #q,
                    #e,
                )
            })
        }
    }
    impl Main {
        /// Convenience function so that tests don't need to import syn
        pub fn parse_syn_ast_from_string(string: String) -> Result<Main, ()> {
            syn::parse_str::<Main>(&string).map_err(|_| ())
        }
    }

    impl Parse for EOI {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            if input.is_empty() {
                Ok(Self)
            } else {
                Err(input.error("Unexpected token"))
            }
        }
    }
    impl ToTokens for EOI {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend(quote! {::jsonpath_ast::ast::EOI})
        }
    }

    impl Parse for Root {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let _ = input.parse::<Token![$]>()?;
            Ok(Root)
        }
    }
    impl ToTokens for Root {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend(quote!(::jsonpath_ast::ast::Root::new()))
        }
    }

    impl ToTokens for WildcardSelector {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend(quote! {::jsonpath_ast::ast::WildcardSelector})
        }
    }

    impl ToTokens for NotOp {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend(quote! {::jsonpath_ast::ast::NotOp})
        }
    }

    impl ParseUtilsExt for Root {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![$])
        }
    }

    impl ToTokens for JPQuery {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self { root, segments } = self;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::JPQuery::new(
                    #root,
                    #segments,
                )
            })
        }
    }

    impl ParseUtilsExt for JPQuery {
        fn peek(input: ParseStream) -> bool {
            Root::peek(input)
        }
    }

    impl ToTokens for Segments {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let mut items = TokenStream::new();
            for item in self.segments.iter() {
                item.to_tokens(&mut items);
                items.extend(quote!(,))
            }
            tokens.extend(quote! {
                ::jsonpath_ast::ast::Segments::new(
                    Vec::from([#items]),
                )
            })
        }
    }

    impl ToTokens for ChildSegment {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                Self::Bracketed(bracketed) => {
                    let mut bracketed_tokens = TokenStream::new();
                    bracketed.to_tokens(&mut bracketed_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::ChildSegment::Bracketed(#bracketed_tokens)
                    });
                }
                Self::WildcardOrShorthand(dot, wildcard_or_shorthand) => {
                    let mut dot_tokens = TokenStream::new();
                    let mut wildcard_or_shorthand_tokens = TokenStream::new();
                    dot.to_tokens(&mut dot_tokens);
                    wildcard_or_shorthand.to_tokens(&mut wildcard_or_shorthand_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::ChildSegment::WildcardOrShorthand(
                            #dot_tokens,
                            #wildcard_or_shorthand_tokens
                        )
                    });
                }
            }
        }
    }

    impl ToTokens for WildcardSelectorOrMemberNameShorthand {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                Self::WildcardSelector(wildcard) => {
                    let mut wildcard_tokens = TokenStream::new();
                    wildcard.to_tokens(&mut wildcard_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::WildcardSelectorOrMemberNameShorthand::WildcardSelector(#wildcard_tokens)
                    });
                }
                Self::MemberNameShorthand(shorthand) => {
                    let mut shorthand_tokens = TokenStream::new();
                    shorthand.to_tokens(&mut shorthand_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::WildcardSelectorOrMemberNameShorthand::MemberNameShorthand(#shorthand_tokens)
                    });
                }
            }
        }
    }

    impl ToTokens for MemberNameShorthand {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let name = &self.name;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::MemberNameShorthand::new(
                    #name.to_string()
                )
            });
        }
    }

    impl ToTokens for BracketedSelection {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let mut selectors_tokens = TokenStream::new();
            self.selectors.to_tokens(&mut selectors_tokens);
            tokens.extend(quote! {
                ::jsonpath_ast::ast::BracketedSelection::new(
                    Default::default(),
                    #selectors_tokens
                )
            });
        }
    }

    impl<T: Parse, P: Parse> ToTokens for PestWithIgnoredPunctuation<T, P>
    where
        T: ToTokens,
        P: ToTokens,
    {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let mut items = TokenStream::new();
            for item in self.0.iter() {
                item.to_tokens(&mut items);
                items.extend(quote!(,))
            }
            tokens.extend(quote! {
                ::jsonpath_ast::ast::PestIgnoredPunctuated::new(::syn::punctuated::Punctuated::from_iter(Vec::from([#items])))
            });
        }
    }

    impl ToTokens for DescendantSegment {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                Self::BracketedSelection(bracketed) => {
                    let mut bracketed_tokens = TokenStream::new();
                    bracketed.to_tokens(&mut bracketed_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::DescendantSegment::BracketedSelection(#bracketed_tokens)
                    });
                }
                Self::WildcardSelector(wildcard) => {
                    let mut wildcard_tokens = TokenStream::new();
                    wildcard.to_tokens(&mut wildcard_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::DescendantSegment::WildcardSelector(#wildcard_tokens)
                    });
                }
                Self::MemberNameShorthand(shorthand) => {
                    let mut shorthand_tokens = TokenStream::new();
                    shorthand.to_tokens(&mut shorthand_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::DescendantSegment::MemberNameShorthand(#shorthand_tokens)
                    });
                }
            }
        }
    }

    impl ToTokens for Segment {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                Self::Child(child) => {
                    let mut child_tokens = TokenStream::new();
                    child.to_tokens(&mut child_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::Segment::new_child(#child_tokens)
                    });
                }
                Self::Descendant(_, _, descendant) => {
                    let mut descendant_tokens = TokenStream::new();
                    descendant.to_tokens(&mut descendant_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::Segment::new_descendant(Default::default(), Default::default(), #descendant_tokens)
                    });
                }
            }
        }
    }
    impl ParseUtilsExt for Segment {
        fn peek(input: ParseStream) -> bool {
            ChildSegment::peek(input) || DescendantSegment::peek(input)
        }
    }

    impl ParseUtilsExt for ChildSegment {
        fn peek(input: ParseStream) -> bool {
            input.peek(token::Bracket) || input.peek(Token![.])
        }
    }

    impl ParseUtilsExt for BracketedSelection {
        fn peek(input: ParseStream) -> bool {
            input.peek(token::Bracket)
        }
    }

    // A named rule exists for this, so it's easier to let the FromPest automatically generate and
    //     just harvest the wildcard token manually in syn
    impl Parse for WildcardSelector {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            input.parse::<Token![*]>().map(|_| WildcardSelector)
        }
    }

    impl ParseUtilsExt for WildcardSelector {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![*])
        }
    }

    impl ParseUtilsExt for MemberNameShorthand {
        fn peek(input: ParseStream) -> bool {
            input.peek(syn::Ident) || input.peek(Token![_])
        }
    }

    pub fn validate_member_name_shorthand(input: ParseStream) -> Result<String, syn::Error> {
        // Special case where syn treats a lone underscore as a token, not an ident
        if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            return Ok("_".to_string());
        }
        let ident = input.parse::<Ident>()?;
        match JSPathParser::parse(Rule::member_name_shorthand, &ident.to_string()) {
            Ok(_) => Ok(ident.to_string()),
            Err(e) => Err(syn::Error::new(ident.span(), e.to_string())),
        }
    }

    impl ParseUtilsExt for DescendantSegment {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![.]) && input.peek2(Token![.])
        }
    }

    impl ParseUtilsExt for Selector {
        fn peek(input: ParseStream) -> bool {
            WildcardSelector::peek(input)
                || (input.peek(Token![:]) || input.peek2(Token![:]))
                || JSInt::peek(input)
                || FilterSelector::peek(input)
                || JSString::peek(input)
        }
    }

    impl ToTokens for Selector {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                Self::WildcardSelector(wildcard) => {
                    let mut wildcard_tokens = TokenStream::new();
                    wildcard.to_tokens(&mut wildcard_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::Selector::new_wildcard_selector(#wildcard_tokens)
                    });
                }
                Self::SliceSelector(slice) => {
                    let mut slice_tokens = TokenStream::new();
                    slice.to_tokens(&mut slice_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::Selector::new_slice_selector(#slice_tokens)
                    });
                }
                Self::IndexSelector(index) => {
                    let mut index_tokens = TokenStream::new();
                    index.to_tokens(&mut index_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::Selector::new_index_selector(#index_tokens)
                    });
                }
                Self::FilterSelector(filter) => {
                    let mut filter_tokens = TokenStream::new();
                    filter.to_tokens(&mut filter_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::Selector::new_filter_selector(#filter_tokens)
                    });
                }
                Self::NameSelector(name) => {
                    let mut name_tokens = TokenStream::new();
                    name.to_tokens(&mut name_tokens);
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::Selector::new_name_selector(#name_tokens)
                    });
                }
            }
        }
    }

    impl ToTokens for SliceSelector {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self(start, _, stop, step) = self;
            let repr_start = match start {
                Some(some_start) => quote! {Some(#some_start)},
                None => quote! {None},
            };
            let repr_stop = match stop {
                Some(some_stop) => quote! {Some(#some_stop)},
                None => quote! {None},
            };
            let repr_step = match step {
                Some(some_step) => quote! {Some(#some_step)},
                None => quote! {None},
            };
            tokens.extend(quote! {
                ::jsonpath_ast::ast::SliceSelector::new(
                    #repr_start,
                    Default::default(),
                    #repr_stop,
                    #repr_step,
                )
            })
        }
    }

    impl ToTokens for SliceStart {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self(_0) = self;
            tokens.extend(quote!(::jsonpath_ast::ast::SliceStart::new(#_0)));
        }
    }
    impl ToTokens for SliceEnd {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self(_0) = self;
            tokens.extend(quote!(::jsonpath_ast::ast::SliceEnd::new(#_0)));
        }
    }

    impl ToTokens for SliceStep {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self(_, _0) = self;
            tokens.extend(quote!(::jsonpath_ast::ast::SliceStep::new(Default::default(), #_0)));
        }
    }

    impl ToTokens for IndexSelector {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self(_0) = self;
            tokens.extend(quote!(::jsonpath_ast::ast::IndexSelector::new(#_0)));
        }
    }

    impl ToTokens for FilterSelector {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self { q, expr } = self;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::FilterSelector::new(
                    #q,
                    #expr,
                )
            });
        }
    }

    impl ToTokens for NameSelector {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self(_0) = self;
            tokens.extend(quote!(::jsonpath_ast::ast::NameSelector::new(#_0)));
        }
    }

    impl ToTokens for LogicalExpr {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self { ands } = self;
            tokens.extend(quote!( ::jsonpath_ast::ast::LogicalExpr::new( #ands ) ));
        }
    }
    impl ToTokens for LogicalExprAnd {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self { atoms } = self;
            tokens.extend(quote!( ::jsonpath_ast::ast::LogicalExprAnd::new( #atoms ) ));
        }
    }

    impl ToTokens for AtomExpr {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend(match self {
                AtomExpr::ParenExpr(inner) => {
                    quote! { ::jsonpath_ast::ast::AtomExpr::new_paren_expr(#inner) }
                }
                AtomExpr::CompExpr(inner) => {
                    quote! { ::jsonpath_ast::ast::AtomExpr::new_comp_expr(#inner) }
                }
                AtomExpr::TestExpr(inner) => {
                    quote! { ::jsonpath_ast::ast::AtomExpr::new_test_expr(#inner) }
                }
            });
        }
    }

    impl ToTokens for ParenExpr {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            #[allow(unused_variables)]
            let Self {
                not_op,
                paren,
                expr,
            } = self;
            tokens.extend(quote! {
            ::jsonpath_ast::ast::ParenExpr::new(
                #not_op,
                Default::default(),
                #expr
            )});
        }
    }
    impl ToTokens for CompExpr {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self { left, op, right } = self;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::CompExpr::new(
                    #left,
                    #op,
                    #right
                )
            });
        }
    }

    impl ToTokens for Comparable {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                Comparable::Literal(inner) => {
                    quote!( new_literal(#inner) )
                }
                Comparable::SingularQuery(inner) => {
                    quote!( new_singular_query(#inner) )
                }
                Comparable::FunctionExpr(inner) => {
                    quote!( new_function_expr(#inner) )
                }
            };
            tokens.extend(quote!(::jsonpath_ast::ast::Comparable::#variant));
        }
    }

    impl ToTokens for Literal {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                Literal::Number(inner) => {
                    quote!(new_number(#inner))
                }
                Literal::String(inner) => {
                    quote!(new_string(#inner))
                }
                Literal::Bool(inner) => {
                    quote!(new_bool(#inner))
                }
                Literal::Null(inner) => {
                    quote!(new_null(#inner))
                }
            };
            tokens.extend(quote!(::jsonpath_ast::ast::Literal::#variant))
        }
    }

    impl ToTokens for Number {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                Number::Int(inner) => {
                    quote!(new_int(#inner))
                }
                Number::Float(inner) => {
                    quote!(new_float(#inner))
                }
            };
            tokens.extend(quote!(::jsonpath_ast::ast::Number::#variant))
        }
    }

    impl ToTokens for SingularQuery {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                SingularQuery::RelSingularQuery(inner) => {
                    quote!(new_rel_singular_query(#inner))
                }
                SingularQuery::AbsSingularQuery(inner) => {
                    quote!(new_abs_singular_query(#inner))
                }
            };
            tokens.extend(quote!(::jsonpath_ast::ast::SingularQuery::#variant ))
        }
    }

    // impl ToTokens for FunctionName {
    //     fn to_tokens(&self, tokens: &mut TokenStream) {
    //         // tokens.extend(quote! {
    //         //     ::jsonpath_ast::ast::FunctionName::new(
    //         //         ::proc_macro2::Ident::new("function_name", ::proc_macro2::Span::call_site())
    //         //     )
    //         // });
    //         let variant = match self {
    //             // Literal::Number(inner) => {
    //             //     quote!(new_number(#inner))
    //             // }
    //             FunctionName::Length(_) => { quote!(new_length(Default::default())) }
    //             FunctionName::Value(_) => { quote!(new_value(Default::default())) }
    //             FunctionName::Count(_) => { quote!(new_count(Default::default())) }
    //             FunctionName::Search(_) => { quote!(new_search(Default::default())) }
    //             FunctionName::Match(_) => { quote!(new_match(Default::default())) }
    //             FunctionName::In(_) => { quote!(new_in(Default::default())) }
    //             FunctionName::Nin(_) => { quote!(new_nin(Default::default())) }
    //             FunctionName::NoneOf(_) => { quote!(new_none_of(Default::default())) }
    //             FunctionName::AnyOf(_) => { quote!(new_any_of(Default::default())) }
    //             FunctionName::SubsetOf(_) => { quote!(new_subset_of(Default::default())) }
    //         };
    //         tokens.extend(quote!(::jsonpath_ast::ast::FunctionName::#variant))
    //     }
    // }

    impl ToTokens for FunctionExpr {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                FunctionExpr::ReturnsValue(v) => { quote!(new_returns_value(#v)) }
                FunctionExpr::ReturnsLogical(v) => { quote!(new_returns_logical(#v)) }
                FunctionExpr::ReturnsNodes(v) => { quote!(new_returns_nodes(#v)) }
            };
            tokens.extend(quote! {
                ::jsonpath_ast::ast::FunctionExpr::#variant
            });
        }
    }

    impl ToTokens for ReturnsValue {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                ReturnsValue::Length(FnCallOneArg{arg, ..}) => { quote!(new_length(Default::default(), #arg)) }
                ReturnsValue::Value(FnCallOneArg{arg, ..}) => { quote!(new_value(Default::default(), #arg)) }
                ReturnsValue::Count(FnCallOneArg{arg, ..}) => { quote!(new_count(Default::default(), #arg)) }
            };
            tokens.extend(quote! {
                ::jsonpath_ast::ast::ReturnsValue::#variant
            });
        }
    }

    impl ToTokens for ReturnsLogical {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                ReturnsLogical::Search(FnCallTwoArg{arg1, arg2, ..}) => {quote!(new_search(Default::default(), #arg1, Default::default(), #arg2))}
                ReturnsLogical::Match(FnCallTwoArg{arg1, arg2, ..}) => {quote!(new_match(Default::default(), #arg1, Default::default(), #arg2))}
                ReturnsLogical::In(FnCallTwoArg{arg1, arg2, ..}) => {quote!(new_in(Default::default(), #arg1, Default::default(), #arg2))}
                ReturnsLogical::Nin(FnCallTwoArg{arg1, arg2, ..}) => {quote!(new_nin(Default::default(), #arg1, Default::default(), #arg2))}
                ReturnsLogical::NoneOf(FnCallTwoArg{arg1, arg2, ..}) => {quote!(new_none_of(Default::default(), #arg1, Default::default(), #arg2))}
                ReturnsLogical::AnyOf(FnCallTwoArg{arg1, arg2, ..}) => {quote!(new_any_of(Default::default(), #arg1, Default::default(), #arg2))}
                ReturnsLogical::SubsetOf(FnCallTwoArg{arg1, arg2, ..}) => {quote!(new_subset_of(Default::default(), #arg1, Default::default(), #arg2))}
            };
            tokens.extend(quote! {
                ::jsonpath_ast::ast::ReturnsLogical::#variant
            });
        }
    }

    impl ToTokens for ReturnsNodes {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            // let variant = match self {
            // };
            tokens.extend(quote! {
                compile_error!("No functions return nodes type yet, if this is no longer true update the ToTokens impl for ReturnsNodes")
            });
        }
    }

    impl ToTokens for ValueType {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                ValueType::Literal(v) => {quote!(new_literal(#v))}
                ValueType::SingularQuery(v) => {quote!(new_singular_query(#v))}
                ValueType::ValueFunction(v) => {quote!(new_value_function(#v))}
            };
            tokens.extend(quote! {
                ::jsonpath_ast::ast::ValueType::#variant
            });
        }
    }

    impl ToTokens for NodesType {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                NodesType::SubQuery(v) => {quote!(new_sub_query(#v))}
                NodesType::NodesFunction(v) => {quote!(new_nodes_function(#v))}
            };
            tokens.extend(quote! {
                ::jsonpath_ast::ast::NodesType::#variant
            });
        }
    }

    impl ToTokens for CompOp {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                CompOp::Eq(_) => {
                    quote!(new_eq)
                }
                CompOp::Ne(_) => {
                    quote!(new_ne)
                }
                CompOp::Le(_) => {
                    quote!(new_le)
                }
                CompOp::Ge(_) => {
                    quote!(new_ge)
                }
                CompOp::Lt(_) => {
                    quote!(new_lt)
                }
                CompOp::Gt(_) => {
                    quote!(new_gt)
                }
            };
            tokens.extend(quote!(::jsonpath_ast::ast::CompOp::#variant(Default::default())));
        }
    }

    impl ToTokens for RelQuery {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self { curr, segments } = self;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::RelQuery::new(
                    #curr,
                    #segments
                )
            });
        }
    }

    impl ToTokens for RelSingularQuery {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            #[allow(unused_variables)]
            let Self { curr, segments } = self;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::RelSingularQuery::new(
                    Default::default(),
                    #segments
                )
            });
        }
    }

    impl ToTokens for AbsSingularQuery {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            #[allow(unused_variables)]
            let Self { root, segments } = self;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::RelSingularQuery::new(
                    Default::default(),
                    #segments
                )
            });
        }
    }

    impl ToTokens for SingularQuerySegments {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let mut out = TokenStream::new();
            for segment in self.segments.iter() {
                out.extend(quote!(#segment,));
            }
            tokens
                .extend(quote!(::jsonpath_ast::ast::SingularQuerySegments::new(Vec::from([#out]))));
        }
    }

    impl ToTokens for SingularQuerySegment {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                Self::NameSegment(segment) => {
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::SingularQuerySegment::new_name_segment(#segment)
                    });
                }
                Self::IndexSegment(segment) => {
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::SingularQuerySegment::new_index_segment(#segment)
                    });
                }
            }
        }
    }

    impl ToTokens for NameSegment {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                Self::BracketedName(name) => {
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::NameSegment::BracketedName(#name)
                    });
                }
                Self::DotName(_dot, shorthand) => {
                    tokens.extend(quote! {
                        ::jsonpath_ast::ast::NameSegment::DotName(
                            Default::default(),
                            #shorthand
                        )
                    });
                }
            }
        }
    }

    impl ToTokens for BracketName {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            #[allow(unused_variables)]
            let Self { bracket, name } = self;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::BracketName {
                    bracket: Default::default(),
                    name: #name,
                }
            });
        }
    }

    impl ToTokens for IndexSegment {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            #[allow(unused_variables)]
            let Self { bracket, index } = self;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::IndexSegment {
                    bracket: Default::default(),
                    index: #index,
                }
            });
        }
    }

    // impl ToTokens for FunctionArgument {
    //     fn to_tokens(&self, tokens: &mut TokenStream) {
    //         match self {
    //             Self::Literal(literal) => {
    //                 let mut literal_tokens = TokenStream::new();
    //                 literal.to_tokens(&mut literal_tokens);
    //                 tokens.extend(quote! {
    //                     ::jsonpath_ast::ast::FunctionArgument::Literal(#literal_tokens)
    //                 });
    //             }
    //             Self::Test(test) => {
    //                 let mut test_tokens = TokenStream::new();
    //                 test.to_tokens(&mut test_tokens);
    //                 tokens.extend(quote! {
    //                     ::jsonpath_ast::ast::FunctionArgument::Test(#test_tokens)
    //                 });
    //             }
    //             Self::LogicalExpr(expr) => {
    //                 let mut expr_tokens = TokenStream::new();
    //                 expr.to_tokens(&mut expr_tokens);
    //                 tokens.extend(quote! {
    //                     ::jsonpath_ast::ast::FunctionArgument::LogicalExpr(#expr_tokens)
    //                 });
    //             }
    //         }
    //     }
    // }

    impl ToTokens for Test {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let variant = match self {
                Test::RelQuery(inner) => {
                    quote!(new_rel_query(#inner))
                }
                Test::JPQuery(inner) => {
                    quote!(new_jp_query(#inner))
                }
                Test::FunctionExpr(inner) => {
                    quote!(new_function_expr(#inner))
                }
            };
            tokens.extend(quote!(::jsonpath_ast::ast::Test::#variant));
        }
    }

    impl ToTokens for TestExpr {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let Self { not_op, test } = self;
            let repr_not = match not_op {
                Some(not_op) => quote! {Some(#not_op)},
                None => quote! {None},
            };
            tokens.extend(quote! {
                ::jsonpath_ast::ast::TestExpr::new(
                    #repr_not,
                    #test
                )
            });
        }
    }

    impl ParseUtilsExt for JSString {
        fn peek(input: ParseStream) -> bool {
            input.peek(syn::LitStr) || input.peek(syn::LitChar)
        }
    }
    /// Validates a JSONPath string literal according to RFC 9535
    /// Control characters (U+0000 through U+001F and U+007F) are not allowed unescaped
    /// in string literals, whether single-quoted or double-quoted
    pub(crate) fn validate_js_str(input: ParseStream) -> Result<String, syn::Error> {
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

    impl ParseUtilsExt for SliceSelector {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![:]) || input.peek2(Token![:])
        }
    }
    impl Parse for ParenExpr {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let not_op: Option<NotOp> = if NotOp::peek(input) {
                Some(input.parse()?)
            } else {
                None
            };
            let __paren_backing_token_stream;
            let paren: PestLiteral<token::Paren> =
                syn::parenthesized!(__paren_backing_token_stream in input ).into();
            let expr: LogicalExpr = __paren_backing_token_stream.parse()?;
            Ok(ParenExpr {
                not_op,
                paren,
                expr,
            })
        }
    }

    impl Parse for NotOp {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            input.parse::<Token![!]>().map(|_| NotOp)
        }
    }

    impl Parse for BracketName {
        fn parse(__input: ParseStream) -> syn::Result<BracketName> {
            let bracket;
            Ok(BracketName {
                bracket: syn::bracketed!(bracket in __input ).into(),
                name: bracket.parse()?,
            })
        }
    }

    impl Parse for IndexSegment {
        fn parse(__input: ParseStream) -> syn::Result<IndexSegment> {
            let bracket;
            Ok(IndexSegment {
                bracket: syn::bracketed!(bracket in __input ).into(),
                index: bracket.parse()?,
            })
        }
    }

    impl ParseUtilsExt for SliceStep {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![:])
        }

        fn maybe_parse(input: ParseStream) -> syn::Result<Option<Self>> {
            if input.peek(Token![:]) {
                let colon = input.parse()?;
                if JSInt::peek(input) {
                    return Ok(Some(Self(colon, input.parse()?)));
                }
            }
            Ok(None)
        }
    }

    impl ParseUtilsExt for SliceStart {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![:]) || input.peek2(Token![:])
        }

        fn maybe_parse(input: ParseStream) -> syn::Result<Option<Self>> {
            if input.peek(Token![:]) {
                return Ok(None);
            } else {
                Ok(Some(Self(input.parse()?)))
            }
        }
    }

    impl ParseUtilsExt for SliceEnd {
        fn peek(input: ParseStream) -> bool {
            JSInt::peek(input)
        }
    }

    impl ParseUtilsExt for FilterSelector {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![?])
        }
    }

    impl ParseUtilsExt for LogicalExpr {
        fn peek(input: ParseStream) -> bool {
            LogicalExprAnd::peek(input)
        }
    }

    impl ParseUtilsExt for LogicalExprAnd {
        fn peek(input: ParseStream) -> bool {
            AtomExpr::peek(input)
        }
    }

    impl ParseUtilsExt for AtomExpr {
        fn peek(input: ParseStream) -> bool {
            ParenExpr::peek(input) || CompExpr::peek(input) || TestExpr::peek(input)
        }
    }

    impl ParseUtilsExt for ParenExpr {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![!]) || input.peek(token::Paren)
        }
    }

    impl ParseUtilsExt for CompExpr {
        fn peek(input: ParseStream) -> bool {
            let fork = input.fork();
            // This is very suboptimal but the only option because at this point in the stream a comp_expr and a test_expr
            //  look identical if they're both functions, IE: $[?match(@, $.regex)] is a test_exp while $[?match(@, $.regex) == true]
            //  is a comp_exp
            fork.parse::<Comparable>().is_ok() && fork.parse::<CompOp>().is_ok()
        }
    }
    impl ParseUtilsExt for TestExpr {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![!]) || Test::peek(input)
        }
    }

    impl ParseUtilsExt for NotOp {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![!])
        }
    }

    impl ParseUtilsExt for Test {
        fn peek(input: ParseStream) -> bool {
            RelQuery::peek(input) || JPQuery::peek(input) || FunctionExpr::peek(input)
        }
    }
    impl ParseUtilsExt for Comparable {
        fn peek(input: ParseStream) -> bool {
            Literal::peek(input) || SingularQuery::peek(input) || FunctionExpr::peek(input)
        }
    }
    impl ParseUtilsExt for JSInt {
        fn peek(input: ParseStream) -> bool {
            input.peek(LitInt)
        }
    }

    impl ToTokens for JSInt {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let value = self.0;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::JSInt::new(#value)
            });
        }
    }

    impl ToTokens for JSString {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let value = &self.0;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::JSString::new(#value.to_string())
            });
        }
    }

    impl ToTokens for Bool {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let value = self.0;
            tokens.extend(quote! {
                ::jsonpath_ast::ast::Bool::new(#value)
            });
        }
    }

    impl ToTokens for Null {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend(quote! {
                ::jsonpath_ast::ast::Null::new(Default::Default())
            });
        }
    }

    /// Only used by syn
    pub fn validate_js_int(input: ParseStream) -> Result<i64, syn::Error> {
        let lit_int = input.parse::<syn::LitInt>()?;
        let parsed = lit_int.base10_parse::<i64>()?;
        Ok(common_bound_validate(parsed).map_err(|e| syn::Error::new(lit_int.span(), e))?)
    }

    const MAX_VAL: i64 = 9007199254740991; // Maximum safe integer value in JavaScript
    const MIN_VAL: i64 = -9007199254740991; // Minimum safe integer value in JavaScript

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

    // fn function_name_expected_args(name: &FunctionName) -> (String, usize) {
    //     (format!("{:?}", name), match name {
    //         FunctionName::Length(_) | FunctionName::Value(_) | FunctionName::Count(_) => { 1 },
    //         FunctionName::Search(_) | FunctionName::Match(_)
    //         | FunctionName::In(_) | FunctionName::Nin(_)
    //         | FunctionName::NoneOf(_) | FunctionName::AnyOf(_) | FunctionName::SubsetOf(_) => { 2 },
    //     })
    // }
    // impl Parse for FunctionExpr {
    //     fn parse(__input: ParseStream) -> ::syn::Result<Self> {
    //         let paren;
    //         let ret = Self { name: __input.parse()?, paren: syn::parenthesized!(paren in __input ), args: PestWithIgnoredPunctuation::parse_separated_nonempty(&paren)? };
    //         let (func_name, expected_num_args) = function_name_expected_args(&ret.name);
    //         if expected_num_args == ret.args.0.len() {
    //             Ok(ret)
    //         } else {
    //             Err(syn::Error::new(ret.args.span(), format!("Invalid number of arguments for function {}, expected {}", func_name, expected_num_args)))
    //         }
    //     }
    // }

    impl ParseUtilsExt for FunctionExpr {
        fn peek(input: ParseStream) -> bool {
            input.peek(kw::length)
                || input.peek(kw::value)
                || input.peek(kw::count)
                || input.peek(kw::search)
                || input.peek(Token![match])
                || input.peek(Token![in])
                || input.peek(kw::nin)
                || input.peek(kw::none_of)
                || input.peek(kw::any_of)
                || input.peek(kw::subset_of)
        }
    }

    impl ParseUtilsExt for ReturnsValue {
        fn peek(input: ParseStream) -> bool {
            input.peek(kw::value) || input.peek(kw::length) || input.peek(kw::count)
        }
    }

    impl ParseUtilsExt for ReturnsLogical {
        fn peek(input: ParseStream) -> bool {
            input.peek(kw::search)
            | input.peek(Token![match])
            | input.peek(Token![in])
            | input.peek(kw::nin)
            | input.peek(kw::none_of)
            | input.peek(kw::any_of)
            | input.peek(kw::subset_of )
        }
    }

    impl<NameToken, Arg> Parse for FnCallOneArg<NameToken, Arg>
    where
        NameToken: Default + Parse,
        Arg: Parse,
    {
        fn parse(__input: ParseStream) -> syn::Result<Self> { Ok(Self { name: __input.parse()?, arg: __input.parse()? }) }
    }

    impl<NameToken: Default, Arg1, Arg2> Parse for FnCallTwoArg<NameToken, Arg1, Arg2>
    where
        NameToken: Default + Parse,
        Arg1: Parse,
        Arg2: Parse,
    {
        fn parse(__input: ParseStream) -> syn::Result<Self> { Ok(Self { name: __input.parse()?, arg1: __input.parse()?, c: __input.parse()?, arg2: __input.parse()? }) }
    }


    impl ParseUtilsExt for RelQuery {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![@])
        }
    }

    impl ParseUtilsExt for SingularQuery {
        fn peek(input: ParseStream) -> bool {
            RelSingularQuery::peek(input) || AbsSingularQuery::peek(input)
        }
    }

    impl ParseUtilsExt for RelSingularQuery {
        fn peek(input: ParseStream) -> bool {
            input.peek(Token![@])
        }
    }

    impl ParseUtilsExt for AbsSingularQuery {
        fn peek(input: ParseStream) -> bool {
            Root::peek(input)
        }
    }

    impl ParseUtilsExt for SingularQuerySegment {
        fn peek(input: ParseStream) -> bool {
            NameSegment::peek(input) || IndexSegment::peek(input)
        }
    }

    impl ParseUtilsExt for NameSegment {
        fn peek(input: ParseStream) -> bool {
            input.peek(token::Bracket) || input.peek(Token![.])
        }
    }

    impl ParseUtilsExt for IndexSegment {
        fn peek(input: ParseStream) -> bool {
            input.peek(token::Bracket)
        }
    }

    impl ParseUtilsExt for Literal {
        fn peek(input: ParseStream) -> bool {
            Number::peek(input) || JSString::peek(input) || Bool::peek(input) || Null::peek(input)
        }
    }

    pub fn parse_float(input: ParseStream) -> syn::Result<f64> {
        let f = input.parse::<syn::LitFloat>()?;
        Ok(f.base10_parse::<f64>()?)
    }

    impl ParseUtilsExt for Number {
        fn peek(input: ParseStream) -> bool {
            JSInt::peek(input) || input.peek(syn::LitFloat)
        }
    }

    pub fn parse_bool(input: ParseStream) -> Result<bool, syn::Error> {
        let lit_bool = input.parse::<syn::LitBool>()?;
        Ok(lit_bool.value)
    }

    impl ParseUtilsExt for Bool {
        fn peek(input: ParseStream) -> bool {
            input.peek(LitBool)
        }
    }

    impl ParseUtilsExt for Null {
        fn peek(input: ParseStream) -> bool {
            input.peek(kw::null)
        }
    }
}
