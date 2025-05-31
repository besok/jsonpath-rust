use jsonpath_ast::ast::Main;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn json_query(input: TokenStream) -> TokenStream {
    let main = parse_macro_input!(input as Main);
    quote! {#main}.into()
}
