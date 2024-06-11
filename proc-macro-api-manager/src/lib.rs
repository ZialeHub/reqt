extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::Attribute;

#[proc_macro_derive(Authorization, attributes(pagination, filter, sort))]
pub fn authorization_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    impl_authorization_derive(&ast)
}

#[proc_macro_derive(Oauth2, attributes(pagination, filter, sort))]
pub fn oauth2_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_oauth2_derive(&ast)
}

#[proc_macro_derive(Basic, attributes(pagination, filter, sort))]
pub fn basic_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_basic_derive(&ast)
}

#[proc_macro_derive(Bearer, attributes(pagination, filter, sort))]
pub fn bearer_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_bearer_derive(&ast)
}

#[proc_macro_derive(ApiKey, attributes(pagination, filter, sort))]
pub fn apikey_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_apikey_derive(&ast)
}

fn impl_authorization_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization for #name {}
    };
    gen.into()
}
fn impl_oauth2_derive(ast: &syn::DeriveInput) -> TokenStream {
    let pagination = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("pagination"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("RequestPagination").unwrap());
    let filter = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("filter"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("FilterRule").unwrap());
    let sort = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("sort"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("SortRule").unwrap());
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();

                let scopes = self
                    .scopes
                    .iter()
                    .fold(String::new(), |acc, scope| format!("{} {}", acc, scope));
                let mut params = HashMap::new();
                params.insert("grant_type", "client_credentials");
                params.insert("client_id", &self.uid);
                params.insert("client_secret", &self.secret);
                params.insert("scope", &scopes);
                match client
                    .post(&self.auth_endpoint)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .form(&params)
                    .send()
                    .await
                {
                    Ok(response) => {
                        match response.status() {
                            StatusCode::OK
                            | StatusCode::CREATED
                            | StatusCode::ACCEPTED
                            | StatusCode::NO_CONTENT => {}
                            status => return Err(status.into()),
                        }
                        match response.text().await {
                            Ok(response_text) => {
                                let token: TokenResponse =
                                    serde_json::from_str(&response_text).unwrap();
                                Ok(connector.oauth2(token.access_token).build())
                            }
                            Err(e) => Err(ApiError::ResponseToText(e)),
                        }
                    }
                    Err(e) => Err(ApiError::ReqwestExecute(e)),
                }
            }
        }
    };
    gen.into()
}

fn impl_basic_derive(ast: &syn::DeriveInput) -> TokenStream {
    let pagination = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("pagination"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("RequestPagination").unwrap());
    let filter = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("filter"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("FilterRule").unwrap());
    let sort = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("sort"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("SortRule").unwrap());
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();
                let encoded_auth = general_purpose::STANDARD_NO_PAD.encode(format!("{}:{}", &self.login, &self.password));

                Ok(connector.basic(encoded_auth).build())
            }
        }
    };
    gen.into()
}

fn impl_bearer_derive(ast: &syn::DeriveInput) -> TokenStream {
    let pagination = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("pagination"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("RequestPagination").unwrap());
    let filter = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("filter"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("FilterRule").unwrap());
    let sort = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("sort"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("SortRule").unwrap());
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();

                Ok(conector.bearer(&self.secret).build())
            }
        }
    };
    gen.into()
}

fn impl_apikey_derive(ast: &syn::DeriveInput) -> TokenStream {
    let pagination = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("pagination"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("RequestPagination").unwrap());
    let filter = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("filter"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("FilterRule").unwrap());
    let sort = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("sort"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<syn::Type>(&name).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("SortRule").unwrap());
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();

                Ok(conector.apikey(&self.key).build())
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(Pagination)]
pub fn pagination_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_pagination_derive(&ast)
}

#[proc_macro_derive(Filter)]
pub fn filter_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_filter_derive(&ast)
}

#[proc_macro_derive(Sort)]
pub fn sort_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_sort_derive(&ast)
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
            fn pagination(mut self, rule: PaginationRule) -> Self {
                self.pagination = rule;
                self
            }
            fn get_pagination(&self) -> &PaginationRule {
                &self.pagination
            }
            fn current_page(&self) -> usize {
                self.current_page
            }
            fn get_current_page(&self) -> Query {
                Query::new()
            }
            fn get_size(&self) -> Query {
                Query::new()
            }
            fn next(&mut self) {
                self.current_page += 1;
            }
            fn get_next_page(&mut self) -> Query {
                Query::new()
            }
        }
    };
    gen.into()
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
