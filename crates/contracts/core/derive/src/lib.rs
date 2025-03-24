use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(UserData)]
pub fn user_data_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl ::quartz_contract_core::msg::execute::attested::HasUserData for #name {
            fn user_data(&self) -> ::quartz_contract_core::state::UserData {
                ::quartz_contract_core::msg::execute::attested::user_data_json(self)
            }
        }
    };

    TokenStream::from(expanded)
}
