use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(UserError)]
pub fn user_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl UserError for #name {
            fn user_error(&self) -> String {
                self.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}
