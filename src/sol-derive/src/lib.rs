extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(AccountData)]
pub fn derive_account_data(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident;
    let field_sizes = match ast.data {
        syn::Data::Struct(data_struct) => data_struct.fields.into_iter().map(|field| {
            let ty = field.ty;
            quote! { <#ty as DataSize>::SIZE }
        }),
        syn::Data::Enum(_data_enum) => todo!(),
        syn::Data::Union(_data_union) => todo!(),
    };

    let discriminator = {
        let hash = Sha256::digest(format!("account|{}", name.to_string()));
        let bytes = &hash[..8];
        quote! { [#( #bytes ),*] }
    };

    let tt = quote! {
        impl AccountData for #name {
            const SIZE: usize = 8 #(+ #field_sizes)*;
            const DISCRIMINATOR: [u8; 8] = #discriminator;
        }
    };

    tt.into()
}
