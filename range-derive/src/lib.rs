extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// The derive macro #[derive(Range)] is used to implement the Range trait by default for a struct.
/// By default the range trait will not add any range to the Query.
#[proc_macro_derive(Range)]
pub fn range_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_range_derive(&ast)
}

fn impl_range_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Range for #name {
            fn pattern(mut self, pattern: impl ToString) -> Self {
                self
            }

            fn range(mut self, property: impl ToString, min: impl ToString, max: impl ToString) -> Self {
                self
            }
        }
    };
    gen.into()
}
