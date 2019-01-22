extern crate proc_macro;

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

#[proc_macro_derive(SimpleError)]
pub fn simple_error(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_simple_error(&ast)
}

fn impl_simple_error(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Error for #name {
            fn cause(&self) -> Option<&dyn Error> {
                None
            }
        }
    };
    gen.into()
}
