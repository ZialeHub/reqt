extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// The derive macro #[derive(Sort)] is used to implement the Sort trait by default for a struct.\
/// The trait will not add any sorts to the Query by default.
#[proc_macro_derive(Sort)]
pub fn sort_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_sort_derive(&ast)
}

fn impl_sort_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Sort for #name {
            fn sort(mut self, property: impl ToString) -> Self {
                self
            }

            fn sort_with(mut self, property: impl ToString, order: SortOrder) -> Self {
                self
            }

            fn pattern(mut self, pattern: impl ToString) -> Self {
                self
            }
        }
    };
    gen.into()
}
