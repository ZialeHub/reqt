extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Ident, Type, Variant};

/// The derive macro #[derive(Authorization)] is used to implement the Authorization trait by default for a struct.\
/// The trait will not add any authorization to the Api by default.
#[proc_macro_derive(Authorization, attributes(pagination, filter, sort, range))]
pub fn authorization_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    impl_authorization_derive(&ast)
}

/// The derive macro #[derive(Oauth2)] is used to implement the Authorization trait for a struct.\
/// The trait will add OAuth2 authorization to the Api.
#[proc_macro_derive(Oauth2, attributes(pagination, filter, sort, range))]
pub fn oauth2_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_oauth2_derive(&ast)
}

/// The derive macro #[derive(Basic)] is used to implement the Authorization trait for a struct.\
/// The trait will add Basic authorization to the Api.
#[proc_macro_derive(Basic, attributes(pagination, filter, sort, range))]
pub fn basic_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_basic_derive(&ast)
}

/// The derive macro #[derive(Bearer)] is used to implement the Authorization trait for a struct.\
/// The trait will add Bearer authorization to the Api.
#[proc_macro_derive(Bearer, attributes(pagination, filter, sort, range))]
pub fn bearer_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_bearer_derive(&ast)
}

/// The derive macro #[derive(ApiKey)] is used to implement the Authorization trait for a struct.\
/// The trait will add ApiKey authorization to the Api.
#[proc_macro_derive(ApiKey, attributes(pagination, filter, sort, range))]
pub fn apikey_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_apikey_derive(&ast)
}

/// The derive macro #[derive(OIDC)] is used to implement the Authorization trait for a struct.\
/// The trait will add OIDC authorization to the Api.
#[proc_macro_derive(OIDC, attributes(pagination, filter, sort, range))]
pub fn oidc_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_oidc_derive(&ast)
}

/// The derive macro #[derive(Keycloak)] is used to implement the Authorization trait for a struct.\
/// The trait will add the AuthorizationType authorization to the Api and will use the Keycloak service.
#[proc_macro_derive(Keycloak, attributes(auth_type, pagination, filter, sort, range))]
pub fn keycloak_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_keycloak_derive(&ast)
}

/// Function to parse generic types for the Authorization implementation
/// - Pagination
/// - Filter
/// - Sort
/// - Range
fn get_attribute_types(ast: &syn::DeriveInput) -> (Type, Type, Type, Type) {
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
    (pagination, filter, sort, range)
}

/// Only impl the Authorization trait for the struct, with the default implementation.
fn impl_authorization_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (pagination, filter, sort, range) = get_attribute_types(ast);
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort, #range> for #name {}
    };
    gen.into()
}

/// Impl the Authorization trait for the struct, with the OAuth2 implementation.\
/// The trait accept the pagination, filter, sort and range types as attributes. (Optionals)\
/// We use the AST to find the attributes (pagination, filter, sort and range) and parse them to the correct type.\
/// If the attribute is not found, we use the default type.
fn impl_oauth2_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (pagination, filter, sort, range) = get_attribute_types(ast);
    let token_struct_name = syn::Ident::new(&format!("{name}TokenOAuth2"), name.span());
    let gen = quote! {
        #[derive(Deserialize)]
        struct #token_struct_name {
            access_token: String,
        }
        impl Authorization<#pagination, #filter, #sort, #range> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort, #range>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();

                let scopes = self
                    .scopes
                    .iter()
                    .fold(String::new(), |acc, scope| format!("{acc} {scope}" scope));
                let mut params = HashMap::new();
                params.insert("grant_type", "client_credentials");
                params.insert("client_id", &self.client_id);
                params.insert("client_secret", &self.client_secret);
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
                                let token: #token_struct_name =
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

/// Impl the Authorization trait for the struct, with the Basic implementation.\
/// The trait accept the pagination, filter, sort and range types as attributes. (Optionals)\
/// We use the AST to find the attributes (pagination, filter, sort and range) and parse them to the correct type.\
/// If the attribute is not found, we use the default type.
fn impl_basic_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (pagination, filter, sort, range) = get_attribute_types(ast);
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

/// Impl the Authorization trait for the struct, with the Bearer implementation.\
/// The trait accept the pagination, filter, sort and range types as attributes. (Optionals)\
/// We use the AST to find the attributes (pagination, filter, sort and range) and parse them to the correct type.\
/// If the attribute is not found, we use the default type.
fn impl_bearer_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (pagination, filter, sort, range) = get_attribute_types(ast);
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort, #range> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort, #range>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();

                Ok(connector.bearer(&self.secret).build())
            }
        }
    };
    gen.into()
}

/// Impl the Authorization trait for the struct, with the ApiKey implementation.\
/// The trait accept the pagination, filter, sort and range types as attributes. (Optionals)\
/// We use the AST to find the attributes (pagination, filter, sort and range) and parse them to the correct type.\
/// If the attribute is not found, we use the default type.
fn impl_apikey_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (pagination, filter, sort, range) = get_attribute_types(ast);
    let gen = quote! {
        impl Authorization<#pagination, #filter, #sort, #range> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort, #range>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();

                Ok(connector.apikey(&self.key).build())
            }
        }
    };
    gen.into()
}

/// Impl the Authorization trait for the struct, with the OIDC implementation.\
/// The trait accept the pagination, filter, sort and range types as attributes. (Optionals)\
/// We use the AST to find the attributes (pagination, filter, sort and range) and parse them to the correct type.\
/// If the attribute is not found, we use the default type.
fn impl_oidc_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (pagination, filter, sort, range) = get_attribute_types(ast);
    let token_struct_name = syn::Ident::new(&format!("{name}TokenOIDC"), name.span());
    let gen = quote! {
        #[derive(Deserialize)]
        struct #token_struct_name {
            access_token: String,
        }
        impl Authorization<#pagination, #filter, #sort, #range> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort, #range>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();

                let scopes = self
                    .scopes
                    .iter()
                    .fold(String::new(), |acc, scope| format!("{acc} {scope}"));
                let mut params = HashMap::new();
                params.insert("grant_type", "client_credentials");
                params.insert("client_id", &self.client_id);
                params.insert("client_secret", &self.client_secret);
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
                                let token: #token_struct_name =
                                    serde_json::from_str(&response_text).unwrap();
                                Ok(connector.oidc(token.access_token).build())
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

/// Impl the Authorization trait for the struct, with the Keycloak implementation.
fn impl_keycloak_derive(ast: &syn::DeriveInput) -> TokenStream {
    let Some(auth_type) = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("auth_type"))
        .and_then(|attr| {
            if let Attribute {
                meta: syn::Meta::List(syn::MetaList { tokens: token, .. }),
                ..
            } = attr
            {
                let name = token.clone().into_iter().next().unwrap().to_string();
                syn::parse_str::<Variant>(&name).ok()
            } else {
                None
            }
        })
    else {
        return quote! {
            compile_error!(
                "You need to provide an AuthenticationType to Keycloak!"
            );
        }
        .into();
    };
    let name = &ast.ident;
    let (pagination, filter, sort, range) = get_attribute_types(ast);
    let auth_variant = auth_type.ident;
    match auth_variant.to_string().as_str() {
        "None" | "Basic" | "Bearer" | "ApiKey" | "OAuth2" => keycloak_authorization_impl(
            auth_variant.to_string(),
            pagination,
            filter,
            sort,
            range,
            name,
        ),
        _ => quote! {
            compile_error!(
                "AuthorizationType must be None, Basic, Bearer, ApiKey or OAuth2 !"
            );
        }
        .into(),
    }
}

/// Impl the Authorization trait for the struct, with the Keycloak implementation.
fn keycloak_authorization_impl(
    auth_type: String,
    pagination: Type,
    filter: Type,
    sort: Type,
    range: Type,
    name: &Ident,
) -> TokenStream {
    let token_struct_name = syn::Ident::new(&format!("{name}TokenKeycloak"), name.span());
    let gen = quote! {
        #[derive(Deserialize)]
        struct #token_struct_name {
            access_token: String,
        }
        impl Authorization<#pagination, #filter, #sort, #range> for #name {
            async fn connect(&self, url: &str) -> Result<Api<#pagination, #filter, #sort, #range>> {
                let connector = ApiBuilder::new(url);
                let client = Client::new();

                let auth_header = format!(
                    "Basic {}",
                    general_purpose::STANDARD_NO_PAD.encode(format!("{}:{}", &self.client_id, &self.client_secret))
                );
                let mut params = HashMap::new();
                params.insert("grant_type", "password");
                params.insert("username", &self.user_login);
                params.insert("password", &self.user_pass);
                match client
                    .post(format!(
                        "{}realms/{}/protocol/openid-connect/token",
                        self.auth_endpoint, self.realm
                    ))
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header("Authorization", auth_header)
                    .form(&params)
                    .send()
                    .await
                {
                    Ok(response) => {
                        log::info!("{:?}", response);
                        match response.status() {
                            StatusCode::OK
                            | StatusCode::CREATED
                            | StatusCode::ACCEPTED
                            | StatusCode::NO_CONTENT => {}
                            status => return Err(status.into()),
                        }
                        match response.text().await {
                            Ok(response_text) => {
                                let token: #token_struct_name =
                                    serde_json::from_str(&response_text).unwrap();
                                Ok(connector.keycloak(match #auth_type {
                                    "None" => AuthorizationType::None,
                                    "Basic" => AuthorizationType::Basic(token.access_token),
                                    "Bearer" => AuthorizationType::Bearer(token.access_token),
                                    "ApiKey" => AuthorizationType::ApiKey(token.access_token),
                                    "OAuth2" => AuthorizationType::OAuth2(token.access_token),
                                    _ => return Err(ApiError::AuthorizationType),
                                }).build())
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
