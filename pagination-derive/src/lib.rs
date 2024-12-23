extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// The derive macro #[derive(Pagination)] is used to implement the Pagination trait by default for a struct.\
/// By default the pagination trait will not add any pagination to the Query.
#[proc_macro_derive(Pagination)]
pub fn pagination_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_pagination_derive(&ast)
}

fn impl_pagination_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Pagination for #name {
            fn size(mut self, size: usize) -> Self {
                self.size = size;
                self
            }
            fn reset(&mut self) {
                self.current_page = 1;
            }
            fn set_pagination(mut self, rule: PaginationRule) -> Self {
                self.pagination = rule;
                self
            }
            fn pagination(&self) -> &PaginationRule {
                &self.pagination
            }
            fn current_page(&self) -> usize {
                self.current_page
            }
            fn get_current_page(&self) -> Query {
                Query::new()
                    .add("page[number]", self.current_page)
                    .add("page[size]", self.size)
            }
            fn get_size(&self) -> Query {
                Query::new().add("page[size]", self.size)
            }
            fn next(&mut self) {
                self.current_page += 1;
            }
            fn get_next_page(&mut self) -> Query {
                self.current_page += 1;
                Query::new()
                    .add("page[number]", self.current_page)
                    .add("page[size]", self.size)
            }
        }
    };
    gen.into()
}
