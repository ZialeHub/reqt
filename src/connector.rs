use std::{fmt::Display, future::Future};

use reqwest::{header::HeaderMap, Method};
use serde::Serialize;

use crate::{error::ApiError, query::Query, request::Request, request_url::RequestUrl};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum PaginationRule {
    #[default]
    None, // Only get the first page
    Fixed(u32), // Limit pages bundling to [u32]
    OneShot,    // Always compute all pages
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum TokenType {
    #[default]
    None,
    // `username:password` into request headers
    // username and password are Base64 encoded
    // `Authorization: Basic bG9sOnNlY3VyZQ==`
    Basic,
    // `token` into request headers
    // `Authorization: Bearer <token>`
    Bearer,
    // `api_key` into request headers
    // `Authorization: Apikey 1234567890abcdef`
    ApiKey,
    // `access_token` into request headers
    // `Authorization: Bearer <access_token>`
    // `refresh_token` into request headers
    // `Authorization: Bearer <refresh_token>`
    OAuth2,
}
impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Basic => write!(f, "Basic"),
            TokenType::ApiKey => write!(f, "ApiKey"),
            TokenType::Bearer => write!(f, "Bearer"),
            TokenType::OAuth2 => write!(f, "Bearer"),
            _ => panic!("TokenType::None is not allowed"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiConnector {
    pub(crate) token_type: TokenType,
    pub(crate) token: String,
    pub(crate) endpoint: String,
    pub(crate) pagination: PaginationRule,
}
impl ApiConnector {
    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = pagination;
        self
    }
}
impl Connector for ApiConnector {
    fn get(&self, route: impl ToString, query: Query) -> Result<Request<()>, ApiError> {
        let mut headers = HeaderMap::new();
        if self.token_type != TokenType::None {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!(
                    "{} {}",
                    self.token_type, self.token
                ))?,
            );
        }
        let url = RequestUrl::new(self.endpoint.clone())
            .route(route.to_string())
            .method(Method::GET)
            .query(query);
        let request = Request::<()>::new(url.method.clone(), url, Some(headers), None)
            .pagination(self.pagination.clone());
        Ok(request)
    }

    fn post<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P>, ApiError> {
        let mut headers = HeaderMap::new();
        if self.token_type != TokenType::None {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!(
                    "{} {}",
                    self.token_type, self.token
                ))?,
            );
        }
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str("application/json")?,
        );
        let url = RequestUrl::new(self.endpoint.clone())
            .route(route.to_string())
            .method(Method::POST)
            .query(query);
        let request = Request::<P>::new(url.method.clone(), url, Some(headers), payload)
            .pagination(self.pagination.clone());
        Ok(request)
    }

    fn put<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P>, ApiError> {
        let mut headers = HeaderMap::new();
        if self.token_type != TokenType::None {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!(
                    "{} {}",
                    self.token_type, self.token
                ))?,
            );
        }
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str("application/json")?,
        );
        let url = RequestUrl::new(self.endpoint.clone())
            .route(route.to_string())
            .method(Method::PUT)
            .query(query);
        let request = Request::<P>::new(url.method.clone(), url, Some(headers), payload)
            .pagination(self.pagination.clone());
        Ok(request)
    }

    fn patch<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P>, ApiError> {
        let mut headers = HeaderMap::new();
        if self.token_type != TokenType::None {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!(
                    "{} {}",
                    self.token_type, self.token
                ))?,
            );
        }
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str("application/json")?,
        );
        let url = RequestUrl::new(self.endpoint.clone())
            .route(route.to_string())
            .method(Method::PATCH)
            .query(query);
        let request = Request::<P>::new(url.method.clone(), url, Some(headers), payload)
            .pagination(self.pagination.clone());
        Ok(request)
    }

    fn delete(&self, route: impl ToString, query: Query) -> Result<Request<()>, ApiError> {
        let mut headers = HeaderMap::new();
        if self.token_type != TokenType::None {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!(
                    "{} {}",
                    self.token_type, self.token
                ))?,
            );
        }
        let url = RequestUrl::new(self.endpoint.clone())
            .route(route.to_string())
            .method(Method::DELETE)
            .query(query);
        let request = Request::<()>::new(url.method.clone(), url, Some(headers), None)
            .pagination(self.pagination.clone());
        Ok(request)
    }
}

/// Trait to implement on your connector structure
/// to allow the use of the `connect` method
pub trait Authentification {
    fn connect(&self, url: &str) -> impl Future<Output = Result<ApiConnector, ApiError>> + Send;
}

pub trait Connector {
    fn get(&self, route: impl ToString, query: Query) -> Result<Request<()>, ApiError>;
    fn post<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P>, ApiError>;
    fn put<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P>, ApiError>;
    fn patch<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P>, ApiError>;
    fn delete(&self, route: impl ToString, query: Query) -> Result<Request<()>, ApiError>;
}
