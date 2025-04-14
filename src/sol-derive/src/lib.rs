extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use sol_gen_common::discriminator::{DiscriminatorGen, HashDiscriminatorGen};
use syn::DeriveInput;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(account_data))]
enum DiscriminatorKind {
    Hash { seed: String, size: u8 },
}

fn derive_account_data_2(
    input: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: DeriveInput = syn::parse2(input)?;

    let attrs: DiscriminatorKind = deluxe::extract_attributes(&mut ast)?;

    let (discriminator, discriminator_size) = {
        let (bytes, size) = match attrs {
            DiscriminatorKind::Hash { seed, size } => (
                HashDiscriminatorGen::discriminator(seed, size as usize),
                size as usize,
            ),
        };
        (quote! { [#( #bytes ),*] }, size)
    };

    let name = ast.ident;
    let field_sizes = match ast.data {
        syn::Data::Struct(data_struct) => data_struct.fields.into_iter().map(|field| {
            let ty = field.ty;
            quote! { <#ty as DataSize>::SIZE }
        }),
        syn::Data::Enum(_data_enum) => todo!(),
        syn::Data::Union(_data_union) => todo!(),
    };

    let tt = quote! {
        impl AccountDataConfig<#discriminator_size> for #name {
            const DATA_SIZE: usize = 0 #(+ #field_sizes)*;
            const DISCRIMINATOR: [u8; #discriminator_size] = #discriminator;
        }
    };

    Ok(tt.into())
}

#[proc_macro_derive(AccountDataConfig, attributes(account_data))]
pub fn derive_account_data(input: TokenStream) -> TokenStream {
    derive_account_data_2(input.into()).unwrap().into()
}
