extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

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
                let mut sort = self.pattern.clone();
                sort = sort.replace("property", &property.to_string());

                self.sorts.push(sort);
                self
            }

            fn sort_with(mut self, property: impl ToString, order: SortOrder) -> Self {
                let mut sort = self.pattern.clone();
                sort = sort.replace("property", &property.to_string());
                sort = sort.replace("order", &order.to_string());

                self.sorts.push(sort);
                self
            }

            fn to_query(&self) -> Query {
                Query::new()
            }

            fn pattern(mut self, pattern: impl ToString) -> Self {
                self.pattern = pattern.to_string();
                self
            }
        }
    };
    gen.into()
}
