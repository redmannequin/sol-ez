extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(AccountRent)]
pub fn derive_account_rent(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident;
    let field_sizes = match ast.data {
        syn::Data::Struct(data_struct) => data_struct.fields.into_iter().map(|field| {
            let ty = field.ty;
            quote! { <#ty as AccountRent>::SIZE }
        }),
        syn::Data::Enum(_data_enum) => todo!(),
        syn::Data::Union(_data_union) => todo!(),
    };

    let tt = quote! {
        impl AccountRent for #name {
            const SIZE: usize = 0 #(+ #field_sizes)*;
        }
    };

    tt.into()
}
