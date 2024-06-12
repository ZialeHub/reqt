extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::Attribute;

/// The derive macro #[derive(Authorization)] is used to implement the Authorization trait by default for a struct.
/// The trait will not add any authorization to the Api by default.
#[proc_macro_derive(Authorization, attributes(pagination, filter, sort, range))]
pub fn authorization_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    impl_authorization_derive(&ast)
}

/// The derive macro #[derive(Oauth2)] is used to implement the Authorization trait for a struct.
/// The trait will add OAuth2 authorization to the Api.
#[proc_macro_derive(Oauth2, attributes(pagination, filter, sort, range))]
pub fn oauth2_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_oauth2_derive(&ast)
}

/// The derive macro #[derive(Basic)] is used to implement the Authorization trait for a struct.
/// The trait will add Basic authorization to the Api.
#[proc_macro_derive(Basic, attributes(pagination, filter, sort, range))]
pub fn basic_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_basic_derive(&ast)
}

/// The derive macro #[derive(Bearer)] is used to implement the Authorization trait for a struct.
/// The trait will add Bearer authorization to the Api.
#[proc_macro_derive(Bearer, attributes(pagination, filter, sort, range))]
pub fn bearer_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_bearer_derive(&ast)
}

/// The derive macro #[derive(ApiKey)] is used to implement the Authorization trait for a struct.
/// The trait will add ApiKey authorization to the Api.
#[proc_macro_derive(ApiKey, attributes(pagination, filter, sort, range))]
pub fn apikey_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_apikey_derive(&ast)
}

/// Only impl the Authorization trait for the struct, with the default implementation.
fn impl_authorization_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization for #name {}
    };
    gen.into()
}

/// Impl the Authorization trait for the struct, with the OAuth2 implementation.
/// The trait accept the pagination, filter, sort and range types as attributes. (Optionals)
/// We use the AST to find the attributes (pagination, filter, sort and range) and parse them to the correct type.
/// If the attribute is not found, we use the default type.
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
    let range = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("range"))
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
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("RangeRule").unwrap());
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort, #range> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort, #range>> {
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

/// Impl the Authorization trait for the struct, with the Basic implementation.
/// The trait accept the pagination, filter, sort and range types as attributes. (Optionals)
/// We use the AST to find the attributes (pagination, filter, sort and range) and parse them to the correct type.
/// If the attribute is not found, we use the default type.
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
    let range = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("range"))
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
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("RangeRule").unwrap());
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort, #range> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort, #range>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();
                let encoded_auth = general_purpose::STANDARD_NO_PAD.encode(format!("{}:{}", &self.login, &self.password));

                Ok(connector.basic(encoded_auth).build())
            }
        }
    };
    gen.into()
}

/// Impl the Authorization trait for the struct, with the Bearer implementation.
/// The trait accept the pagination, filter, sort and range types as attributes. (Optionals)
/// We use the AST to find the attributes (pagination, filter, sort and range) and parse them to the correct type.
/// If the attribute is not found, we use the default type.
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
    let range = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("range"))
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
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("RangeRule").unwrap());
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort, #range> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort, #range>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();

                Ok(conector.bearer(&self.secret).build())
            }
        }
    };
    gen.into()
}

/// Impl the Authorization trait for the struct, with the ApiKey implementation.
/// The trait accept the pagination, filter, sort and range types as attributes. (Optionals)
/// We use the AST to find the attributes (pagination, filter, sort and range) and parse them to the correct type.
/// If the attribute is not found, we use the default type.
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
    let range = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("range"))
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
        .unwrap_or_else(|| syn::parse_str::<syn::Type>("RangeRule").unwrap());
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort, #range> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort, #range>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();

                Ok(conector.apikey(&self.key).build())
            }
        }
    };
    gen.into()
}
