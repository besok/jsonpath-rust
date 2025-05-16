

use proc_macro::TokenStream;
use std::fmt::format;
use jsonpath_ast::ast::{ Main, JPQuery };
use syn::parse_macro_input;
use quote::{quote, ToTokens};

#[proc_macro]
pub fn json_query(input: TokenStream) -> TokenStream {
    let main = parse_macro_input!(input as Main);
    quote! {#main}.into()
}



// #[proc_macro]
// pub fn json_query(input: TokenStream) -> TokenStream {
//     // Pest accepts &str, not TokenStreams
//     let input_as_str = input.to_string();
//     let result: Parsed<JpQuery> = parse_json_path(&*input_as_str);
//     match result {
//         // Confirmed parsing works during compile time, so unwrap path built from the exact same string at run time
//         Ok(_) => {
//             quote!(parse_json_path(#input_as_str).unwrap()).into()
//         }
//         Err(e) => {
//             // If it's a Pest error, make the best possible effort to highlight the correct token
//             if let JsonPathError::PestError(pest_err) = e {
//                 let (start, _finish): (usize, usize) = match pest_err.location {
//                     InputLocation::Pos(pos) => (pos, pos+1),
//                     InputLocation::Span((start, finish)) => (start, finish)
//                 };
//                 let mut stream = input.into_iter();
//                 let first = stream.next();
//                 if let Some(first_tt) = first {
//                     // byte_range() is marked as unreliable when not on nightly, but I have no idea of a better way
//                     let start_span: proc_macro2::Span = proc_macro2::Span::from(first_tt.span());
//                     let start_val = start_span.byte_range().start + start;
//                     let mut err_span: proc_macro2::Span = start_span.clone();
//                     for tt in stream {
//                         let span: Range<usize> = proc_macro2::Span::from(tt.span()).byte_range();
//                         if span.start >= start_val {
//                             err_span = proc_macro2::Span::from(tt.span());
//                             break;
//                         }
//                     }
//                     syn::Error::new(err_span.into(), pest_err).to_compile_error().into()
//                 } else {
//                     syn::Error::new(Span::call_site().into(), "Expected JsonPath, found no input").to_compile_error().into()
//                 }
//             } else {
//                 syn::Error::new(Span::call_site().into(), e).to_compile_error().into()
//             }
//         }
//     }
// }