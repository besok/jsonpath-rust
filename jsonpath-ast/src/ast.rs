pub mod parse {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "../../jsonpath-rust/src/parser/grammar/json_path_9535.pest"]
    pub struct JSPathParser;
}

pub(crate) mod kw {
    // syn::custom_keyword!(in);
    syn::custom_keyword!(nin);
    syn::custom_keyword!(size);
    syn::custom_keyword!(noneOf);
    syn::custom_keyword!(anyOf);
    syn::custom_keyword!(subsetOf);

    syn::custom_keyword!(null);
}

macro_rules! terminating_from_pest {
    ($wrap:ty, $rule:path, $parser:expr) => {
        #[automatically_derived]
        impl<'pest> ::from_pest::FromPest<'pest> for $wrap {
            type Rule = Rule;
            type FatalError = ::from_pest::Void;
            fn from_pest(
                pest: &mut ::from_pest::pest::iterators::Pairs<'pest, Rule>,
            ) -> ::std::result::Result<Self, ::from_pest::ConversionError<::from_pest::Void>> {
                let mut clone = pest.clone();
                let pair = clone.next().ok_or(::from_pest::ConversionError::NoMatch)?;
                if pair.as_rule() == $rule {
                    let mut inner = pair.clone().into_inner();
                    let inner = &mut inner;
                    let this = $parser(pair);
                    if inner.clone().next().is_some() {
                        ::from_pest::log::trace!(
                            "when converting {}, found extraneous {:?}",
                            stringify!($wrap),
                            inner
                        );
                        Err(::from_pest::ConversionError::Extraneous {
                            current_node: stringify!($wrap),
                        })?;
                    }
                    *pest = clone;
                    Ok(this)
                } else {
                    Err(::from_pest::ConversionError::NoMatch)
                }
            }
        }
    };
}

use derive_new::new;
use super::parse::{JSPathParser, Rule};
#[cfg(feature = "compiled-path")]
use crate::syn_parse::parse_impl::{
    parse_bool, parse_float, validate_function_name, validate_js_int, validate_js_str,
    validate_member_name_shorthand, ParseUtilsExt,
};
use from_pest::{ConversionError, FromPest, Void};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_ast::FromPest;
use proc_macro2::Span;
#[cfg(feature = "compiled-path")]
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::token::Bracket;
use syn::{token, Ident, LitBool, Token};
#[cfg(feature = "compiled-path")]
use syn_derive::{Parse, ToTokens};
#[cfg(feature = "compiled-path")]
use proc_macro2::TokenStream;

pub trait KnowsRule {
    const RULE: Rule;
}

#[derive(Debug, new, PartialEq)]
pub struct PestIgnoredPunctuated<T, P>(pub(crate) Punctuated<T, P>);

impl<'pest, T, P> FromPest<'pest> for PestIgnoredPunctuated<T, P>
where
    T: FromPest<'pest, Rule = Rule, FatalError = Void> + KnowsRule,
    P: Default,
{
    type Rule = Rule;
    type FatalError = Void;

    fn from_pest(
        pest: &mut Pairs<'pest, Self::Rule>,
    ) -> Result<Self, ConversionError<Self::FatalError>> {
        let parsed_items = Vec::<T>::from_pest(pest)?;

        Ok(PestIgnoredPunctuated(Punctuated::from_iter(
            parsed_items.into_iter(),
        )))
    }
}

/// Allows for syn to parse things that pest checks but does not store as rules
#[derive(Debug, Default, new, PartialEq)]
pub struct PestLiteralWithoutRule<T>(pub(crate) T);

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

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::main))]
pub struct Main {
    pub(crate) jp_query: JPQuery,
    pub(crate) eoi: EOI,
}

impl Main {
    /// Convenience function to allow tests to not import pest::parser::Parser
    pub fn try_from_pest_parse(str: &str) -> Result<Self, ()> {
        let mut rules = JSPathParser::parse(Rule::main, str).map_err(|_| ())?;
        // *IF* the FromPest implementations are correctly written then Main::from_pest *cannot fail*
        Main::from_pest(&mut rules).map_err(|_| ())
    }
}

#[derive(Debug, new, PartialEq, FromPest)]
#[pest_ast(rule(Rule::EOI))]
pub struct EOI;

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::jp_query))]
pub struct JPQuery {
    pub(crate) root: PestLiteralWithoutRule<Root>,
    pub(crate) segments: Segments,
}

#[derive(Debug, new, PartialEq, Default)]
pub struct Root;

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::segments))]
pub struct Segments {
    #[cfg_attr(feature = "compiled-path", parse(Segment::parse_outer))]
    pub(crate) segments: Vec<Segment>,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::segment))]
pub enum Segment {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = ChildSegment::peek))]
    Child(ChildSegment),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = DescendantSegment::peek))]
    Descendant(DescendantSegment),
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::child_segment))]
pub enum ChildSegment {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = BracketedSelection::peek))]
    Bracketed(BracketedSelection),
    // search for `[` or `.`(must NOT be `..` because that is a descendant segment but syn will parse that as `..` not 2 periods)
    #[cfg_attr(feature = "compiled-path", parse(peek = Token![.]))]
    WildcardOrShorthand(
        PestLiteralWithoutRule<token::Dot>,
        WildcardSelectorOrMemberNameShorthand,
    ),
}

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub struct BracketedSelection {
    #[cfg_attr(feature = "compiled-path", syn(bracketed))]
    pub(crate) arg_bracket: token::Bracket,
    #[cfg_attr(feature = "compiled-path", syn(in = arg_bracket))]
    #[cfg_attr(feature = "compiled-path", parse(|i: ParseStream| PestIgnoredPunctuated::parse_terminated(i)))]
    pub(crate) selectors: PestIgnoredPunctuated<Selector, Token![,]>,
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

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub enum WildcardSelectorOrMemberNameShorthand {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = WildcardSelector::peek))]
    WildcardSelector(WildcardSelector),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = MemberNameShorthand::peek))]
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

#[derive(Debug, new, PartialEq, FromPest)]
#[pest_ast(rule(Rule::wildcard_selector))]
pub struct WildcardSelector;

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub struct MemberNameShorthand {
    #[cfg_attr(feature = "compiled-path", parse(validate_member_name_shorthand))]
    pub(crate) name: String,
}

impl<'pest> from_pest::FromPest<'pest> for MemberNameShorthand {
    type Rule = Rule;
    type FatalError = Void;
    fn from_pest(pest: &mut Pairs<'pest, Rule>) -> Result<Self, ConversionError<Void>> {
        let mut clone = pest.clone();
        let pair = clone.next().ok_or(ConversionError::NoMatch)?;
        if pair.as_rule() == Rule::member_name_shorthand {
            *pest = clone;
            Ok(MemberNameShorthand {
                name: pest.as_str().to_string(),
            })
        } else {
            Err(ConversionError::NoMatch)
        }
    }
}

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub struct JSString(#[cfg_attr(feature = "compiled-path", parse(validate_js_str))] pub(crate) String);

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

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::descendant_segment))]
pub enum DescendantSegment {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = BracketedSelection::peek))]
    BracketedSelection(BracketedSelection),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = WildcardSelector::peek))]
    WildcardSelector(WildcardSelector),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = MemberNameShorthand::peek))]
    MemberNameShorthand(MemberNameShorthand),
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::selector))]
pub enum Selector {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = WildcardSelector::peek))]
    WildcardSelector(WildcardSelector),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = SliceSelector::peek))]
    SliceSelector(SliceSelector),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = JSInt::peek))]
    IndexSelector(JSInt),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = FilterSelector::peek))]
    FilterSelector(FilterSelector),
    // This MUST be the last element to prevent syn::Lit from catching one of the others, it's our "fallback"
    #[cfg_attr(feature = "compiled-path", parse(peek_func = JSString::peek))]
    NameSelector(JSString),
}
impl KnowsRule for Selector {
    const RULE: Rule = Rule::selector;
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::slice_selector))]
pub struct SliceSelector(
    #[cfg_attr(feature = "compiled-path", parse(SliceStart::maybe_parse))] pub(crate) Option<SliceStart>,
    pub(crate) PestLiteralWithoutRule<Token![:]>,
    #[cfg_attr(feature = "compiled-path", parse(SliceEnd::maybe_parse))] pub(crate) Option<SliceEnd>,
    #[cfg_attr(feature = "compiled-path", parse(SliceStep::maybe_parse))] pub(crate) Option<SliceStep>,
);

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

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::step))]
pub struct SliceStep(pub(crate) PestLiteralWithoutRule<Token![:]>, pub(crate) JSInt);

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::start))]
pub struct SliceStart(pub(crate) JSInt);

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::end))]
pub struct SliceEnd(pub(crate) JSInt);

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::filter_selector))]
pub struct FilterSelector {
    pub q: PestLiteralWithoutRule<Token![?]>,
    pub expr: LogicalExpr,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::logical_expr))]
pub struct LogicalExpr {
    pub ands: PestIgnoredPunctuated<LogicalExprAnd, Token![||]>,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::logical_expr_and))]
pub struct LogicalExprAnd {
    pub atoms: PestIgnoredPunctuated<AtomExpr, Token![&&]>,
}
impl KnowsRule for LogicalExprAnd {
    const RULE: Rule = Rule::logical_expr_and;
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::atom_expr))]
pub enum AtomExpr {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = ParenExpr::peek))]
    ParenExpr(ParenExpr),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = CompExpr::peek))]
    CompExpr(CompExpr),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = TestExpr::peek))]
    TestExpr(TestExpr),
}
impl KnowsRule for AtomExpr {
    const RULE: Rule = Rule::atom_expr;
}

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub struct JSInt(#[cfg_attr(feature = "compiled-path", parse(validate_js_int))] pub(crate) i64);

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

// New implementations below LLM STUFF

#[derive(Debug, new, PartialEq, FromPest)]
#[pest_ast(rule(Rule::paren_expr))]
pub struct ParenExpr {
    // #[cfg_attr(feature = "compiled-path", parse(peek_func = NotOp::peek))]
    pub not_op: Option<NotOp>,
    // #[paren]
    pub paren: PestLiteralWithoutRule<token::Paren>,
    // #[inside(paren)]
    pub expr: LogicalExpr,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::comp_expr))]
pub struct CompExpr {
    left: Comparable,
    op: CompOp,
    right: Comparable,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::test_expr))]
pub struct TestExpr {
    #[cfg_attr(feature = "compiled-path", parse(NotOp::maybe_parse))]
    not_op: Option<NotOp>,
    test: Test,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[pest_ast(rule(Rule::not_op))]
pub struct NotOp;

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::test))]
pub enum Test {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = RelQuery::peek))]
    RelQuery(RelQuery),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = JPQuery::peek))]
    JPQuery(JPQuery),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = FunctionExpr::peek))]
    FunctionExpr(FunctionExpr),
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::comparable))]
pub enum Comparable {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = Literal::peek))]
    Literal(Literal),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = SingularQuery::peek))]
    SingularQuery(SingularQuery),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = FunctionExpr::peek))]
    FunctionExpr(FunctionExpr),
}

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub enum CompOp {
    #[cfg_attr(feature = "compiled-path", parse(peek = Token![==]))]
    Eq(Token![==]),
    #[cfg_attr(feature = "compiled-path", parse(peek = Token![!=]))]
    Ne(Token![!=]),
    #[cfg_attr(feature = "compiled-path", parse(peek = Token![<=]))]
    Le(Token![<=]),
    #[cfg_attr(feature = "compiled-path", parse(peek = Token![>=]))]
    Ge(Token![>=]),
    #[cfg_attr(feature = "compiled-path", parse(peek = Token![<]))]
    Lt(Token![<]),
    #[cfg_attr(feature = "compiled-path", parse(peek = Token![>]))]
    Gt(Token![>]),
    // #[cfg_attr(feature = "compiled-path", parse(peek_func = |input: ParseStream| input.peek(syn::token::In)))]
    // In(syn::token::In),
    // #[cfg_attr(feature = "compiled-path", parse(peek_func = |input: ParseStream| input.peek(kw::nin)))]
    // Nin(kw::nin),
    // #[cfg_attr(feature = "compiled-path", parse(peek_func = |input: ParseStream| input.peek(kw::size)))]
    // Size(kw::size),
    // #[cfg_attr(feature = "compiled-path", parse(peek_func = |input: ParseStream| input.peek(kw::noneOf)))]
    // NoneOf(kw::noneOf),
    // #[cfg_attr(feature = "compiled-path", parse(peek_func = |input: ParseStream| input.peek(kw::anyOf)))]
    // AnyOf(kw::anyOf),
    // #[cfg_attr(feature = "compiled-path", parse(peek_func = |input: ParseStream| input.peek(kw::subsetOf)))]
    // SubsetOf(kw::subsetOf),
}
impl KnowsRule for CompOp {
    const RULE: Rule = Rule::comp_op;
}
impl<'pest> FromPest<'pest> for CompOp {
    type Rule = Rule;
    type FatalError = Void;

    fn from_pest(
        pest: &mut Pairs<'pest, Self::Rule>,
    ) -> Result<Self, ConversionError<Self::FatalError>> {
        let mut clone = pest.clone();
        let pair = clone.next().ok_or(ConversionError::NoMatch)?;
        if pair.as_rule() == Self::RULE {
            *pest = clone;
            Ok(match pair.as_str() {
                "==" => Self::Eq(Default::default()),
                "!=" => Self::Ne(Default::default()),
                "<=" => Self::Le(Default::default()),
                ">=" => Self::Ge(Default::default()),
                "<" => Self::Lt(Default::default()),
                ">" => Self::Gt(Default::default()),
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

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub struct FunctionExpr {
    pub(crate) name: FunctionName,
    #[cfg_attr(feature = "compiled-path", syn(parenthesized))]
    pub(crate) paren: token::Paren,
    #[cfg_attr(feature = "compiled-path", syn(in = paren))]
    #[cfg_attr(feature = "compiled-path", parse(|i: ParseStream| PestIgnoredPunctuated::parse_terminated(i)))]
    pub(crate) args: PestIgnoredPunctuated<FunctionArgument, Token![,]>,
}

impl<'pest> from_pest::FromPest<'pest> for FunctionExpr {
    type Rule = Rule;
    type FatalError = Void;
    fn from_pest(pest: &mut Pairs<'pest, Rule>) -> Result<Self, ConversionError<Void>> {
        let mut clone = pest.clone();
        let pair = clone.next().ok_or(ConversionError::NoMatch)?;
        if pair.as_rule() == Rule::function_expr {
            let mut inner = pair.into_inner();
            let inner = &mut inner;
            let this = FunctionExpr {
                name: ::from_pest::FromPest::from_pest(inner)?,
                paren: Default::default(),
                args: FromPest::from_pest(inner)?,
            };
            if inner.clone().next().is_some() {
                Err(ConversionError::Extraneous {
                    current_node: "FunctionExpr",
                })?;
            }
            *pest = clone;
            Ok(this)
        } else {
            Err(ConversionError::NoMatch)
        }
    }
}

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub struct FunctionName {
    #[cfg_attr(feature = "compiled-path", parse(validate_function_name))]
    name: Ident,
}

impl<'pest> FromPest<'pest> for FunctionName {
    type Rule = Rule;
    type FatalError = Void;

    fn from_pest(
        pest: &mut Pairs<'pest, Self::Rule>,
    ) -> Result<Self, ConversionError<Self::FatalError>> {
        let mut clone = pest.clone();
        let pair = clone.next().ok_or(ConversionError::NoMatch)?;
        if pair.as_rule() == Rule::function_name {
            let mut inner = pair.into_inner();
            let inner = &mut inner;
            let this = FunctionName {
                name: Ident::new(inner.to_string().as_str(), Span::call_site()),
            };
            if inner.clone().next().is_some() {
                from_pest::log::trace!(
                    "when converting {}, found extraneous {:?}",
                    stringify!(FunctionName),
                    inner
                );
                Err(ConversionError::Extraneous {
                    current_node: stringify!(FunctionName),
                })?;
            }
            *pest = clone;
            Ok(this)
        } else {
            Err(ConversionError::NoMatch)
        }
    }
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::function_argument))]
pub enum FunctionArgument {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = Literal::peek))]
    Literal(Literal),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = Test::peek))]
    Test(Test),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = LogicalExpr::peek))]
    LogicalExpr(LogicalExpr),
}
impl KnowsRule for FunctionArgument {
    const RULE: Rule = Rule::function_argument;
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::rel_query))]
pub struct RelQuery {
    curr: PestLiteralWithoutRule<Token![@]>,
    segments: Segments,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::singular_query))]
pub enum SingularQuery {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = RelSingularQuery::peek))]
    RelSingularQuery(RelSingularQuery),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = AbsSingularQuery::peek))]
    AbsSingularQuery(AbsSingularQuery),
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::rel_singular_query))]
pub struct RelSingularQuery {
    curr: PestLiteralWithoutRule<Token![@]>,
    segments: SingularQuerySegments,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::abs_singular_query))]
pub struct AbsSingularQuery {
    root: PestLiteralWithoutRule<Root>,
    segments: SingularQuerySegments,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::singular_query_segments))]
pub struct SingularQuerySegments {
    #[cfg_attr(feature = "compiled-path", parse(SingularQuerySegment::parse_outer))]
    segments: Vec<SingularQuerySegment>,
}

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub enum SingularQuerySegment {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = NameSegment::peek))]
    NameSegment(NameSegment),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = IndexSegment::peek))]
    IndexSegment(IndexSegment),
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
                // SingularQueryStatement is NOT actually a rule so pass pest, not inner
                let this = Self::NameSegment(::from_pest::FromPest::from_pest(pest)?);
                Ok(this)
            }
            Rule::index_segment => {
                let mut inner = pair.clone().into_inner();
                let inner = &mut inner;
                // SingularQueryStatement is NOT actually a rule so pass pest, not inner
                let this = Self::IndexSegment(::from_pest::FromPest::from_pest(pest)?);
                Ok(this)
            }
            _ => Err(ConversionError::NoMatch),
        }
    }
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::name_segment))]
pub enum NameSegment {
    #[cfg_attr(feature = "compiled-path", parse(peek = token::Bracket))]
    BracketedName(BracketName),
    #[cfg_attr(feature = "compiled-path", parse(peek = Token![.]))]
    DotName(PestLiteralWithoutRule<Token![.]>, MemberNameShorthand),
}

#[derive(Debug, new, PartialEq, FromPest)]
#[pest_ast(rule(Rule::name_selector))]
pub struct BracketName {
    // #[cfg_attr(feature = "compiled-path", syn(bracketed))]
    pub bracket: PestLiteralWithoutRule<Bracket>,
    // #[cfg_attr(feature = "compiled-path", syn(in = bracket))]
    pub name: JSString,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[pest_ast(rule(Rule::index_segment))]
pub struct IndexSegment {
    // #[cfg_attr(feature = "compiled-path", syn(bracketed))]
    pub bracket: PestLiteralWithoutRule<Bracket>,
    // #[cfg_attr(feature = "compiled-path", syn(in = bracket))]
    pub index: JSInt,
}

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::literal))]
pub enum Literal {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = Number::peek))]
    Number(Number),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = JSString::peek))]
    String(JSString),
    #[cfg_attr(feature = "compiled-path", parse(peek = LitBool))]
    Bool(Bool),
    #[cfg_attr(feature = "compiled-path", parse(peek_func = Null::peek))]
    Null(Null),
}

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub enum Number {
    #[cfg_attr(feature = "compiled-path", parse(peek_func = JSInt::peek))]
    Int(JSInt),
    #[cfg_attr(feature = "compiled-path", parse(peek = syn::LitFloat))]
    Float(#[cfg_attr(feature = "compiled-path", parse(parse_float))] f64),
}

impl<'pest> FromPest<'pest> for Number {
    type Rule = Rule;
    type FatalError = Void;
    fn from_pest(
        pest: &mut Pairs<'pest, Self::Rule>,
    ) -> Result<Self, ConversionError<Self::FatalError>> {
        let mut clone = pest.clone();
        let pair = clone.next().ok_or(ConversionError::NoMatch)?;

        if pair.as_rule() == Rule::number {
            let mut inner = pair.into_inner();
            let inner = &mut inner;

            let this = if inner.clone().count() == 1 {
                let pair = inner.next().unwrap();
                if pair.as_rule() == Rule::int {
                    let value = pair.as_str().parse::<i64>().expect("int");
                    Ok(Self::Int(JSInt(value)))
                } else {
                    Err(ConversionError::NoMatch)
                }
            } else {
                let mut number_str = String::new();
                for pair in &mut *inner {
                    number_str.push_str(pair.as_str());
                }

                let value = number_str.parse::<f64>().expect("float");
                Ok(Self::Float(value))
            }?;
            if inner.next().is_some() {
                from_pest::log::trace!(
                    "when converting {}, found extraneous {:?}",
                    stringify!(FunctionName),
                    inner
                );
                Err(ConversionError::Extraneous {
                    current_node: stringify!(FunctionName),
                })?;
            }
            *pest = clone;
            Ok(this)
        } else {
            Err(ConversionError::NoMatch)
        }
    }
}

#[derive(Debug, new, PartialEq)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
pub struct Bool(#[cfg_attr(feature = "compiled-path", parse(parse_bool))]pub(crate) bool);

terminating_from_pest!(Bool, Rule::bool, |inner: Pair<Rule>| Bool(
    inner.as_str().parse::<bool>().expect("bool")
));

#[derive(Debug, new, PartialEq, FromPest)]
#[cfg_attr(feature = "compiled-path", derive(Parse))]
#[pest_ast(rule(Rule::null))]
pub struct Null(pub(crate) PestLiteralWithoutRule<kw::null>);
