extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// The derive macro #[derive(Filter)] is used to implement the Filter trait by default for a struct.\
/// The trait will not add any filters to the Query by default.
#[proc_macro_derive(Filter)]
pub fn filter_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_filter_derive(&ast)
}

fn impl_filter_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Filter for #name {
            fn filter<T: IntoIterator>(mut self, property: impl ToString, value: T) -> Self
            where
                T::Item: ToString,
            {
                self
            }

            fn filter_with<T: IntoIterator>(mut self, property: impl ToString, filter: impl ToString, value: T) -> Self
            where
                T::Item: ToString,
            {
                self
            }

            fn pattern(mut self, pattern: impl ToString) -> Self {
                self
            }
        }
    };
    gen.into()
}
