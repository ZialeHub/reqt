extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// The derive macro #[derive(Filter)] is used to implement the Filter trait by default for a struct.
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
                let mut filter = self.pattern.clone();
                let mut values = String::new();
                filter = filter.replace("property", &property.to_string());

                for v in value.into_iter() {
                    values.push_str(&v.to_string());
                    values.push(',');
                }
                values.pop();
                self.filters.push((filter, values));
                self
            }

            fn filter_with<T: IntoIterator>(mut self, property: impl ToString, filter: impl ToString, value: T) -> Self
            where
                T::Item: ToString,
            {
                let mut filters = self.pattern.clone();
                let mut values = String::new();
                filters = filters.replace("property", &property.to_string());
                filters = filters.replace("filter", &filter.to_string());

                for v in value.into_iter() {
                    values.push_str(&v.to_string());
                    values.push(',');
                }
                values.pop();
                self.filters.push((filters, values));
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
