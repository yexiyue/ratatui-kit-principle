use proc_macro::TokenStream;
use quote::ToTokens;

mod element;

#[proc_macro]
pub fn element(input: TokenStream) -> TokenStream {
    let element = syn::parse_macro_input!(input as element::ParsedElement);
    element.to_token_stream().into()
}
