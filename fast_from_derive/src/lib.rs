#![feature(extern_crate_item_prelude)]

extern crate proc_macro;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(BadRequest)]
pub fn fast_from(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_from_macro(&ast)
} 

fn impl_from_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl From<#name> for BadRequestError {
            fn from(err: #name) -> Self {
                BadRequestError(Box::new(err))
            }
        }
    };
    gen.into()
}