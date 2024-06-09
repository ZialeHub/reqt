use std::{fmt::Display, future::Future};

use reqwest::{header::HeaderMap, Method};
use serde::Serialize;

use crate::{
    error::Result,
    pagination::{Pagination, PaginationRule, RequestPagination},
    prelude::ApiBuilder,
    query::Query,
    request::Request,
    request_url::RequestUrl,
};

/// Authorization type to be used in the API
#[derive(Debug, Clone, Default, PartialEq)]
pub enum AuthorizationType {
    #[default]
    None,
    // `username:password` into request headers
    // username and password are Base64 encoded
    // `Authorization: Basic bG9sOnNlY3VyZQ==`
    Basic(String),
    // `token` into request headers
    // `Authorization: Bearer <token>`
    Bearer(String),
    // `api_key` into request headers
    // `X-API-Key: 1234567890abcdef`
    ApiKey(String),
    // `access_token` into request headers
    // `Authorization: Bearer <access_token>`
    // `refresh_token` into request headers
    // `Authorization: Bearer <refresh_token>`
    OAuth2(String),
}

impl AuthorizationType {
    /// Set the Authorization header value for the request
    ///
    /// # Arguments
    /// * `headers` - A mutable reference to the request headers
    pub fn header_value(&self, headers: &mut HeaderMap) -> Result<()> {
        match self {
            AuthorizationType::None => {}
            AuthorizationType::ApiKey(_) => {
                headers.insert(
                    "X-API-Key",
                    reqwest::header::HeaderValue::from_str(&self.to_string())?,
                );
            }
            _ => {
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&self.to_string())?,
                );
            }
        }

        Ok(())
    }
}

impl Display for AuthorizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthorizationType::Basic(token) => write!(f, "Basic {}", token),
            AuthorizationType::ApiKey(token) => write!(f, "{}", token),
            AuthorizationType::Bearer(token) | AuthorizationType::OAuth2(token) => {
                write!(f, "Bearer {}", token)
            }
            _ => panic!("TokenType::None is not allowed"),
        }
    }
}

/// Structure to build requests to the API
///
/// # Parameters
/// * `T` - Pagination type to be used in the request
///
/// # Attributes
/// * authorization - Authorization type to be used in the request
/// * endpoint - API endpoint to be used in the request
/// * pagination - Pagination type to be used in the request
#[derive(Debug, Clone)]
pub struct Api<T: Pagination = RequestPagination> {
    pub(crate) authorization: AuthorizationType,
    pub(crate) endpoint: String,
    pub(crate) pagination: T,
}

impl<T: Pagination> Api<T> {
    pub fn pagination(mut self, pagination: PaginationRule) -> Self {
        self.pagination = self.pagination.pagination(pagination);
        self
    }

    pub fn token(&self) -> String {
        self.authorization.to_string()
    }
}

impl<T: Pagination> Connector<T> for Api<T> {
    fn get(&self, route: impl ToString, query: Query) -> Result<Request<(), T>> {
        let mut headers = HeaderMap::new();

        self.authorization.header_value(&mut headers)?;

        let url = RequestUrl::new(&self.endpoint)
            .route(route.to_string())
            .method(Method::GET)
            .query(query);

        let request = Request::<(), T>::new(
            url.method.clone(),
            url,
            Some(headers),
            None,
            self.pagination.clone(),
        )
        .pagination(self.pagination.get_pagination().clone());

        Ok(request)
    }

    fn post<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P, T>> {
        let mut headers = HeaderMap::new();

        self.authorization.header_value(&mut headers)?;

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str("application/json")?,
        );

        let url = RequestUrl::new(&self.endpoint)
            .route(route.to_string())
            .method(Method::POST)
            .query(query);

        let request = Request::<P, T>::new(
            url.method.clone(),
            url,
            Some(headers),
            payload,
            self.pagination.clone(),
        )
        .pagination(self.pagination.get_pagination().clone());

        Ok(request)
    }

    fn put<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P, T>> {
        let mut headers = HeaderMap::new();

        self.authorization.header_value(&mut headers)?;

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str("application/json")?,
        );

        let url = RequestUrl::new(&self.endpoint)
            .route(route.to_string())
            .method(Method::PUT)
            .query(query);

        let request = Request::<P, T>::new(
            url.method.clone(),
            url,
            Some(headers),
            payload,
            self.pagination.clone(),
        )
        .pagination(self.pagination.get_pagination().clone());

        Ok(request)
    }

    fn patch<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P, T>> {
        let mut headers = HeaderMap::new();

        self.authorization.header_value(&mut headers)?;

        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_str("application/json")?,
        );

        let url = RequestUrl::new(&self.endpoint)
            .route(route.to_string())
            .method(Method::PATCH)
            .query(query);

        let request = Request::<P, T>::new(
            url.method.clone(),
            url,
            Some(headers),
            payload,
            self.pagination.clone(),
        )
        .pagination(self.pagination.get_pagination().clone());

        Ok(request)
    }

    fn delete(&self, route: impl ToString, query: Query) -> Result<Request<(), T>> {
        let mut headers = HeaderMap::new();

        self.authorization.header_value(&mut headers)?;

        let url = RequestUrl::new(&self.endpoint)
            .route(route.to_string())
            .method(Method::DELETE)
            .query(query);

        let request = Request::<(), T>::new(
            url.method.clone(),
            url,
            Some(headers),
            None,
            self.pagination.clone(),
        )
        .pagination(self.pagination.get_pagination().clone());

        Ok(request)
    }
}

/// Trait to implement on your connector structure
/// to allow the use of the `connect` method
pub trait Authorization<T: Pagination + Default + Send = RequestPagination> {
    fn connect(&self, url: &str) -> impl Future<Output = Result<Api<T>>> + Send {
        async move { Ok(ApiBuilder::new(url, T::default()).build()) }
    }
}

/// Trait to implement on your connector structure
/// to allow the use of the `get`, `post`, `put`, `patch`, and `delete` methods
pub trait Connector<T: Pagination> {
    fn get(&self, route: impl ToString, query: Query) -> Result<Request<(), T>>;
    fn post<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P, T>>;
    fn put<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P, T>>;
    fn patch<P: Serialize + Clone>(
        &self,
        route: impl ToString,
        query: Query,
        payload: Option<P>,
    ) -> Result<Request<P, T>>;
    fn delete(&self, route: impl ToString, query: Query) -> Result<Request<(), T>>;
}
