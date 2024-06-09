extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Authorization)]
pub fn authorization_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_authorization_derive(&ast)
}

#[proc_macro_derive(Oauth2)]
pub fn oauth2_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_oauth2_derive(&ast)
}

#[proc_macro_derive(Basic)]
pub fn basic_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_basic_derive(&ast)
}

#[proc_macro_derive(Bearer)]
pub fn bearer_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_bearer_derive(&ast)
}

#[proc_macro_derive(ApiKey)]
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
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization for #name {
            async fn connect(&self, url: &str) -> Result<Api<RequestPagination>> {
                let pagination = RequestPagination::default();
                let connector = ApiBuilder::new(url, pagination);
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
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization for #name {
            async fn connect(&self, url: &str) -> Result<Api<RequestPagination>> {
                let pagination = RequestPagination::default();
                let connector = ApiBuilder::new(url, pagination);
                let client = Client::new();
                let encoded_auth = general_purpose::STANDARD_NO_PAD.encode(format!("{}:{}", &self.login, &self.password));

                Ok(connector.basic(encoded_auth).build())
            }
        }
    };
    gen.into()
}

fn impl_bearer_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization for #name {
            async fn connect(&self, url: &str) -> Result<Api<RequestPagination>> {
                let pagination = RequestPagination::default();
                let connector = ApiBuilder::new(url, pagination);
                let client = Client::new();

                Ok(conector.bearer(&self.secret).build())
            }
        }
    };
    gen.into()
}

fn impl_apikey_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Authorization for #name {
            async fn connect(&self, url: &str) -> Result<Api<RequestPagination>> {
                let pagination = RequestPagination::default();
                let connector = ApiBuilder::new(url, pagination);
                let client = Client::new();

                Ok(conector.apikey(&self.key).build())
            }
        }
    };
    gen.into()
}
